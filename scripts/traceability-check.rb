#!/usr/bin/env ruby
# frozen_string_literal: true

# Traceability check (change-control support for spec/traceability.md and
# the spec 09 "definition of verified"). Standard library only.
#
# Enforced:
#   1. Every rule code in the clinical YAML is exercised by at least one
#      test artifact (Rust suites, web suites or the clinical scenario set).
#   2. Every requirement ID referenced from implementation sources exists in
#      the specification — a comment cannot cite a requirement that does not
#      exist.
# Reported (informational): per-prefix coverage of specification
# requirement IDs referenced from implementation and tests.

require "find"

ROOT = File.expand_path("..", __dir__)

def read_all(paths)
  paths.map { |path| File.read(path, encoding: "UTF-8") }.join("\n")
end

def collect(globs, exclude: [])
  globs.flat_map { |glob| Dir.glob(File.join(ROOT, glob)) }
       .reject { |path| exclude.any? { |fragment| path.include?(fragment) } }
       .select { |path| File.file?(path) }
       .sort
end

failures = []

# ---- 1. Rule codes must be exercised by tests ------------------------------
clinical_yaml = File.read(File.join(ROOT, "spec/clinical/nice-cg98-2023-10-31.1.yaml"))
rule_codes = clinical_yaml.scan(/code:\s*([A-Z][A-Z0-9_]+)/).flatten.uniq.sort

test_corpus = read_all(
  collect(
    [
      "crates/*/tests/**/*.rs",
      "apps/*/tests/**/*.rs",
      "web/tests/**/*.ts",
      "validation/*.yaml",
    ],
  ),
)

untested = rule_codes.reject { |code| test_corpus.include?(code) }
untested.each { |code| failures << "rule code #{code} is not exercised by any test artifact" }

# ---- 2. Referenced requirement IDs must exist in the specification ---------
ID_PATTERN = /\b(?:PRD|CLIN|DATA|API|WEB|SEC|SAFE|OPS|TEST)-\d{3}\b/

spec_corpus = read_all(collect(["spec/**/*.md", "spec/**/*.yaml"]))
defined_ids = spec_corpus.scan(ID_PATTERN).uniq

implementation_files = collect(
  [
    "crates/**/*.rs",
    "apps/**/*.rs",
    "web/app/**/*.{ts,tsx,css}",
    "web/components/**/*.{ts,tsx}",
    "web/lib/**/*.ts",
    "web/tests/**/*.ts",
    "web/scripts/**/*.mjs",
    "infrastructure/**/*.tf",
    "infrastructure/**/*.md",
    "scripts/**/*",
    "Dockerfile",
  ],
  exclude: ["node_modules", "target/", "web/out", "generated"],
)

referenced = Hash.new { |hash, key| hash[key] = [] }
implementation_files.each do |path|
  File.read(path, encoding: "UTF-8").scan(ID_PATTERN).each do |id|
    referenced[id] << path unless referenced[id].include?(path)
  end
rescue ArgumentError
  next # skip non-UTF-8 binaries
end

referenced.keys.sort.each do |id|
  next if defined_ids.include?(id)

  failures << "requirement #{id} is referenced (#{referenced[id].first.sub("#{ROOT}/", "")}) but not defined in the specification"
end

# ---- Coverage report -------------------------------------------------------
puts "Traceability report"
puts "  rule codes: #{rule_codes.size} defined, #{rule_codes.size - untested.size} exercised by tests"
%w[PRD CLIN DATA API WEB SEC SAFE OPS TEST].each do |prefix|
  defined = defined_ids.select { |id| id.start_with?("#{prefix}-") }
  cited = defined.select { |id| referenced.key?(id) }
  puts format("  %-5s %3d defined, %3d referenced from implementation/tests", prefix, defined.size, cited.size)
end

if failures.empty?
  puts "Traceability check passed."
  exit 0
end

puts "\nTraceability check FAILED:"
failures.each { |failure| puts "  - #{failure}" }
exit 1
