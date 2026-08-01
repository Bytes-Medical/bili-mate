"use client";

// Intended purpose, sources, rule-pack version, licence and safety
// information (spec 06 routes). Legal wording comes from the server so this
// page can never ship a divergent copy (spec 04 /v1/legal).

import { useEffect, useState } from "react";

import { apiClient, type LegalNotices, type RulePackMetadata } from "@/lib/api/client";

export default function AboutPage() {
  const [legal, setLegal] = useState<LegalNotices | null>(null);
  const [pack, setPack] = useState<RulePackMetadata | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [legalResult, packResult] = await Promise.all([
          apiClient.GET("/v1/legal"),
          apiClient.GET("/v1/guidelines/active"),
        ]);
        if (cancelled) return;
        if (legalResult.data) setLegal(legalResult.data);
        if (packResult.data) setPack(packResult.data);
        if (!legalResult.data && !packResult.data) setFailed(true);
      } catch {
        if (!cancelled) setFailed(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <>
      <p className="eyebrow">About</p>
      <h1>Intended purpose and sources</h1>
      {failed && (
        <div className="notice" role="note">
          The service could not be reached, so the current legal wording and rule-pack version
          cannot be shown. Try again when you are online.
        </div>
      )}
      {legal && (
        <>
          <h2>Intended purpose</h2>
          <p>{legal.intended_purpose}</p>
          <p>
            <strong>Intended users:</strong> {legal.intended_users}
          </p>
          <div className="notice" role="note">
            {legal.professional_use_warning} {legal.local_pathology_warning}
          </div>
          <h2>NICE content</h2>
          <p>{legal.nice_attribution}</p>
          <p>{legal.non_endorsement}</p>
          <h2>Privacy</h2>
          <p>{legal.privacy_summary}</p>
        </>
      )}
      {pack && (
        <div className="panel panel-filled">
          <p className="eyebrow">Active clinical rule pack</p>
          <p className="mono small">
            {pack.id} — {pack.guideline_title}
            <br />
            Source updated {pack.source_updated_on} · status {pack.status}
            <br />
            Content SHA-256 {pack.content_sha256}
          </p>
          <ul className="small">
            {pack.sources.map((source) => (
              <li key={source.id}>
                <a href={source.url}>{source.id}</a> (retrieved {source.retrieved_on})
              </li>
            ))}
          </ul>
        </div>
      )}
    </>
  );
}
