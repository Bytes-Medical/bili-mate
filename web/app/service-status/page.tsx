"use client";

// Availability and current release metadata, with no patient information
// (spec 06 routes).

import { useEffect, useState } from "react";

import { apiClient, API_BASE_URL, type RulePackMetadata } from "@/lib/api/client";

type Status = "checking" | "ready" | "unavailable";

export default function ServiceStatusPage() {
  const [status, setStatus] = useState<Status>("checking");
  const [pack, setPack] = useState<RulePackMetadata | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const ready = await fetch(`${API_BASE_URL}/health/ready`, { cache: "no-store" });
        if (cancelled) return;
        setStatus(ready.ok ? "ready" : "unavailable");
        const metadata = await apiClient.GET("/v1/guidelines/active");
        if (!cancelled && metadata.data) setPack(metadata.data);
      } catch {
        if (!cancelled) setStatus("unavailable");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <>
      <p className="eyebrow">Service status</p>
      <h1>Availability and release information</h1>
      <div className="panel" aria-live="polite">
        <p className="eyebrow">Clinical evaluation service</p>
        <p className="value">
          {status === "checking" && "Checking…"}
          {status === "ready" && "Available"}
          {status === "unavailable" && "Not available"}
        </p>
        {status === "unavailable" && (
          <p>
            No clinical result can be produced while the service is unavailable. Use your locally
            approved procedure for assessing and managing neonatal jaundice.
          </p>
        )}
      </div>
      {pack && (
        <div className="panel panel-filled">
          <p className="eyebrow">Current release</p>
          <p className="mono small">
            Rule pack {pack.id} ({pack.status})
            <br />
            Source updated {pack.source_updated_on}
          </p>
        </div>
      )}
    </>
  );
}
