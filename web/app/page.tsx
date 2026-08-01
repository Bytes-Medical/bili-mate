"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";

import AckGate from "@/components/AckGate";
import { hasAcknowledged } from "@/lib/ack";

export default function HomePage() {
  const router = useRouter();
  const [acknowledged, setAcknowledged] = useState<boolean | null>(null);
  const [showGate, setShowGate] = useState(false);

  useEffect(() => {
    setAcknowledged(hasAcknowledged());
  }, []);

  return (
    <>
      <p className="eyebrow">Clinical decision support — demonstration service</p>
      <h1>Assessment and management support for neonatal jaundice</h1>
      <p>
        Bili Mate supports registered UK healthcare professionals assessing jaundice in babies from
        birth to less than 28 days of age. It presents NICE CG98 bilirubin treatment thresholds up
        to and including 14 days and the recommendations that follow from the facts you enter.
      </p>
      <div className="notice" role="note">
        Visual inspection alone cannot estimate the bilirubin level, and changes in skin colour can
        be harder to see in darker skin tones. Every result needs review against the baby, local
        pathology advice and local policy.
      </div>
      <p>
        Your entries stay in this browser while you work: dates and times never leave the page, and
        the server keeps no record of the assessment after it responds.
      </p>

      {showGate || acknowledged === false ? (
        <AckGate onAcknowledged={() => router.push("/assessment")} />
      ) : (
        <p>
          {acknowledged ? (
            <Link className="btn" href="/assessment">
              Start an assessment
            </Link>
          ) : (
            <button type="button" className="btn" onClick={() => setShowGate(true)}>
              Start an assessment
            </button>
          )}
        </p>
      )}

      <div className="rule-above">
        <h2>What Bili Mate does not do</h2>
        <p>
          It does not diagnose the cause of jaundice, order treatment, store patient records or
          work offline. It contains no artificial intelligence and accepts no patient identifiers.
          Results are calculated by the server from the current approved rule pack; this browser
          performs no clinical calculation.
        </p>
      </div>
    </>
  );
}
