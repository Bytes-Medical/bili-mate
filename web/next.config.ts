import type { NextConfig } from "next";

// ADR-009: static export only. No API routes, server actions or server
// rendering of clinical data can exist in this deployment model.
const nextConfig: NextConfig = {
  output: "export",
  images: { unoptimized: true },
  // The API origin is fixed at build time; no clinical input ever appears in
  // a URL (spec 06 information architecture).
  env: {
    NEXT_PUBLIC_API_BASE_URL: process.env.NEXT_PUBLIC_API_BASE_URL ?? "https://api.bili-mate.uk",
  },
};

export default nextConfig;
