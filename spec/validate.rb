# frozen_string_literal: true

require "bigdecimal"
require "date"
require "json"
require "psych"
require "time"
require "uri"
require "yaml"

ROOT = File.expand_path(__dir__)
OPENAPI_PATH = File.join(ROOT, "openapi.yaml")
RULE_PACK_PATH = File.join(ROOT, "clinical", "nice-cg98-2023-10-31.1.yaml")

errors = []

def local_ref(document, reference)
  return nil unless reference.start_with?("#/")

  reference.delete_prefix("#/").split("/").reduce(document) do |node, part|
    key = part.gsub("~1", "/").gsub("~0", "~")
    node.is_a?(Hash) ? node[key] : nil
  end
end

def duplicate_yaml_keys(path)
  found = []
  visit = nil
  visit = lambda do |node, pointer|
    if node.is_a?(Psych::Nodes::Mapping)
      seen = {}
      node.children.each_slice(2) do |key, value|
        label = key.respond_to?(:value) ? key.value : "<complex-key>"
        found << "#{pointer}: duplicate YAML key #{label.inspect}" if seen[label]
        seen[label] = true
        visit.call(value, "#{pointer}/#{label}")
      end
    elsif node.respond_to?(:children) && node.children
      node.children.each_with_index { |child, index| visit.call(child, "#{pointer}/#{index}") }
    end
  end
  visit.call(Psych.parse_file(path), path)
  found
end

def type_matches?(value, type)
  case type
  when "null" then value.nil?
  when "object" then value.is_a?(Hash)
  when "array" then value.is_a?(Array)
  when "string" then value.is_a?(String)
  when "integer" then value.is_a?(Integer)
  when "number" then value.is_a?(Numeric)
  when "boolean" then value == true || value == false
  else false
  end
end

def validate_schema(value, schema, document, pointer, found)
  if schema.key?("$ref")
    target = local_ref(document, schema["$ref"])
    if target.nil?
      found << "#{pointer}: unresolved schema reference #{schema["$ref"]}"
    else
      validate_schema(value, target, document, pointer, found)
    end
    return
  end

  if schema.key?("anyOf")
    branch_errors = schema["anyOf"].map do |branch|
      candidate = []
      validate_schema(value, branch, document, pointer, candidate)
      candidate
    end
    found << "#{pointer}: does not satisfy anyOf (#{branch_errors.map(&:length).join("/")} errors)" if branch_errors.none?(&:empty?)
    return
  end

  types = Array(schema["type"]).compact
  unless types.empty?
    unless types.any? { |type| type_matches?(value, type) }
      found << "#{pointer}: expected #{types.join(" or ")}, got #{value.class}"
      return
    end
  end

  found << "#{pointer}: value is not the declared const" if schema.key?("const") && value != schema["const"]
  found << "#{pointer}: value is outside the declared enum" if schema.key?("enum") && !schema["enum"].include?(value)

  if value.is_a?(Hash)
    Array(schema["required"]).each do |key|
      found << "#{pointer}: missing required property #{key.inspect}" unless value.key?(key)
    end
    properties = schema.fetch("properties", {})
    value.each do |key, child|
      if properties.key?(key)
        validate_schema(child, properties[key], document, "#{pointer}/#{key}", found)
      elsif schema["additionalProperties"] == false
        found << "#{pointer}: undeclared property #{key.inspect}"
      elsif schema["additionalProperties"].is_a?(Hash)
        validate_schema(child, schema["additionalProperties"], document, "#{pointer}/#{key}", found)
      end
    end
  elsif value.is_a?(Array)
    found << "#{pointer}: fewer than minItems" if schema["minItems"] && value.length < schema["minItems"]
    found << "#{pointer}: more than maxItems" if schema["maxItems"] && value.length > schema["maxItems"]
    if schema["uniqueItems"] && value.map { |item| JSON.generate(item) }.uniq.length != value.length
      found << "#{pointer}: array items are not unique"
    end
    value.each_with_index do |child, index|
      validate_schema(child, schema["items"], document, "#{pointer}/#{index}", found) if schema["items"]
    end
  elsif value.is_a?(String)
    found << "#{pointer}: shorter than minLength" if schema["minLength"] && value.length < schema["minLength"]
    found << "#{pointer}: longer than maxLength" if schema["maxLength"] && value.length > schema["maxLength"]
    found << "#{pointer}: does not match pattern" if schema["pattern"] && !Regexp.new(schema["pattern"]).match?(value)
    begin
      case schema["format"]
      when "date" then Date.iso8601(value)
      when "date-time" then Time.iso8601(value)
      when "uuid"
        raise ArgumentError unless value.match?(/\A[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\z/i)
      when "uri"
        uri = URI.parse(value)
        raise URI::InvalidURIError unless uri.absolute?
      end
    rescue ArgumentError, URI::InvalidURIError
      found << "#{pointer}: invalid #{schema["format"]} format"
    end
  elsif value.is_a?(Numeric)
    found << "#{pointer}: below minimum" if schema["minimum"] && value < schema["minimum"]
    found << "#{pointer}: above maximum" if schema["maximum"] && value > schema["maximum"]
    if schema["multipleOf"]
      dividend = BigDecimal(value.to_s)
      divisor = BigDecimal(schema["multipleOf"].to_s)
      found << "#{pointer}: not a multiple of #{schema["multipleOf"]}" unless dividend.remainder(divisor).zero?
    end
  end
end

[OPENAPI_PATH, RULE_PACK_PATH].each do |path|
  begin
    errors.concat(duplicate_yaml_keys(path))
    YAML.parse_file(path)
  rescue Psych::SyntaxError => e
    errors << "#{path}: invalid YAML: #{e.message}"
  end
end

openapi = YAML.safe_load(File.read(OPENAPI_PATH), permitted_classes: [Date], aliases: true)
rule_pack_document = YAML.safe_load(File.read(RULE_PACK_PATH), permitted_classes: [Date], aliases: true)

refs = []
walk = nil
walk = lambda do |value|
  case value
  when Hash
    value.each do |key, child|
      refs << child if key == "$ref" && child.is_a?(String) && child.start_with?("#/")
      walk.call(child)
    end
  when Array
    value.each { |child| walk.call(child) }
  end
end
walk.call(openapi)
refs.uniq.each do |reference|
  errors << "#{OPENAPI_PATH}: unresolved reference #{reference}" unless local_ref(openapi, reference)
end

operation_ids = openapi.fetch("paths").values.flat_map do |path_item|
  path_item.values.map { |operation| operation["operationId"] if operation.is_a?(Hash) }.compact
end
operation_ids.group_by(&:itself).each do |operation_id, occurrences|
  errors << "#{OPENAPI_PATH}: duplicate operationId #{operation_id}" if occurrences.length > 1
end

example_schemas = {
  /-request\.json\z/ => "EvaluationRequest",
  /-response\.json\z/ => "EvaluationResponse",
  /-problem\.json\z/ => "Problem"
}
Dir[File.join(ROOT, "examples", "*.json")].sort.each do |path|
  begin
    value = JSON.parse(File.read(path))
    schema_name = example_schemas.find { |pattern, _name| pattern.match?(path) }&.last
    if schema_name
      validate_schema(value, openapi.dig("components", "schemas", schema_name), openapi, path, errors)
    else
      errors << "#{path}: no example schema mapping"
    end
  rescue JSON::ParserError => e
    errors << "#{path}: invalid JSON: #{e.message}"
  end
end

Dir[File.join(ROOT, "**", "*.md")].sort.each do |path|
  File.read(path).scan(/\[[^\]]*\]\(([^)]+)\)/).each do |match|
    target = match.first
    next if target.match?(/\A(?:https?:|mailto:|#)/)

    target = target[1..-2] if target.start_with?("<") && target.end_with?(">")
    relative = target.split("#", 2).first
    next if relative.empty?

    resolved = File.expand_path(relative, File.dirname(path))
    errors << "#{path}: missing local link target #{target}" unless File.exist?(resolved)
  end
end

requirement_sources = Dir[File.join(ROOT, "**", "*.{md,yaml}")].sort.reject do |path|
  path == File.join(ROOT, "traceability.md")
end
requirement_text = requirement_sources.map { |path| File.read(path) }.join("\n")
requirement_prefixes = %w[PRD CLIN DATA API WEB SEC SAFE OPS TEST]
requirement_prefixes.each do |prefix|
  numbers = requirement_text.scan(/\b#{prefix}-(\d{3})\b/).flatten.map(&:to_i).uniq.sort
  expected = (1..numbers.max).to_a
  missing = expected - numbers
  errors << "#{prefix}: missing requirement IDs #{missing.join(", ")}" unless missing.empty?
end

traceability = File.read(File.join(ROOT, "traceability.md"))
trace_ids = traceability.scan(/\b(#{requirement_prefixes.join("|")})-(\d{3})\b/).map do |prefix, number|
  format("%s-%03d", prefix, number.to_i)
end
traceability.scan(/\b(#{requirement_prefixes.join("|")})-(\d{3})[–-]\1-(\d{3})\b/).each do |prefix, first, last|
  (first.to_i..last.to_i).each { |number| trace_ids << format("%s-%03d", prefix, number) }
end
defined_ids = requirement_text.scan(/\b(#{requirement_prefixes.join("|")})-(\d{3})\b/).map do |prefix, number|
  format("%s-%03d", prefix, number.to_i)
end.uniq
untraced_requirements = defined_ids - trace_ids.uniq
errors << "Untraced requirements: #{untraced_requirements.join(", ")}" unless untraced_requirements.empty?

rule_pack = rule_pack_document.fetch("rule_pack")
rules = rule_pack.fetch("rules")
%w[code order].each do |field|
  rules.group_by { |rule| rule[field] }.each do |value, occurrences|
    errors << "#{RULE_PACK_PATH}: duplicate rule #{field} #{value.inspect}" if occurrences.length > 1
  end
end
rules.each do |rule|
  errors << "#{RULE_PACK_PATH}: #{rule["code"]} has no source_refs" if Array(rule["source_refs"]).empty?
end

untraced = rules.map { |rule| rule.fetch("code") }.reject { |code| traceability.include?("`#{code}`") }
errors << "#{RULE_PACK_PATH}: untraced rules #{untraced.join(", ")}" unless untraced.empty?

if rule_pack["status"] == "active"
  errors << "#{RULE_PACK_PATH}: active pack needs an author" if Array(rule_pack["authors"]).empty?
  errors << "#{RULE_PACK_PATH}: active pack needs two clinical reviewers" if Array(rule_pack["clinical_reviewers"]).length < 2
  errors << "#{RULE_PACK_PATH}: active pack needs a Clinical Safety Officer" if rule_pack["clinical_safety_officer"].nil?
  rule_pack.fetch("sources").each do |source|
    errors << "#{RULE_PACK_PATH}: active source #{source["id"]} needs sha256" unless source["sha256"].to_s.match?(/\A[a-f0-9]{64}\z/)
  end
end

if errors.empty?
  puts "Specification validation passed"
  puts "  OpenAPI references: #{refs.uniq.length}"
  puts "  JSON examples: #{Dir[File.join(ROOT, "examples", "*.json")].length}"
  puts "  Clinical rules: #{rules.length}"
  puts "  Markdown files: #{Dir[File.join(ROOT, "**", "*.md")].length}"
else
  warn errors.join("\n")
  exit 1
end
