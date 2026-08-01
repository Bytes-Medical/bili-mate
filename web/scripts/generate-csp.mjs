// CSP hash generation (ADR-009, SEC-016): Next.js static export emits inline
// bootstrap scripts, and the deployed CSP must allow exactly those scripts by
// hash — never with 'unsafe-inline'. This step hashes every inline <script>
// in the exported HTML, writes the deployable policy to out/csp.txt and
// fails the build if an inline event handler or javascript: URL would be
// uncovered by a hash-based policy.

import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const OUT_DIR = new URL("../out", import.meta.url).pathname;
const API_ORIGIN = new URL(process.env.NEXT_PUBLIC_API_BASE_URL ?? "https://api.bili-mate.uk")
  .origin;

function htmlFiles(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) {
      files.push(...htmlFiles(path));
    } else if (entry.endsWith(".html")) {
      files.push(path);
    }
  }
  return files;
}

const hashes = new Set();
let failures = 0;

for (const file of htmlFiles(OUT_DIR)) {
  const html = readFileSync(file, "utf8");
  // Inline scripts (no src=) need a hash each.
  for (const match of html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/gi)) {
    const body = match[1];
    if (body.trim().length === 0) continue;
    const digest = createHash("sha256").update(body, "utf8").digest("base64");
    hashes.add(`'sha256-${digest}'`);
  }
  // Inline event handlers cannot be allowed by hash; they would force
  // 'unsafe-inline', which SEC-016 prohibits.
  if (/<[a-z][^>]*\son[a-z]+\s*=/i.test(html)) {
    console.error(`inline event handler found in ${file}`);
    failures += 1;
  }
  if (/javascript:/i.test(html)) {
    console.error(`javascript: URL found in ${file}`);
    failures += 1;
  }
}

if (failures > 0) {
  console.error("CSP generation failed: the export contains script content no hash policy can cover.");
  process.exit(1);
}

const scriptSrc = ["'self'", ...[...hashes].sort()].join(" ");
const policy = [
  `default-src 'self'`,
  `script-src ${scriptSrc}`,
  `style-src 'self' 'unsafe-inline'`,
  `img-src 'self' data:`,
  `font-src 'self'`,
  `connect-src 'self' ${API_ORIGIN}`,
  `object-src 'none'`,
  `base-uri 'none'`,
  `frame-ancestors 'none'`,
  `form-action 'self'`,
].join("; ");

writeFileSync(join(OUT_DIR, "csp.txt"), `${policy}\n`);
console.log(
  `CSP generated with ${hashes.size} inline script hash(es); connect-src allows ${API_ORIGIN}.`,
);
console.log("Deploy this policy as the Content-Security-Policy header for the static site.");
