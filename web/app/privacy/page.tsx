export const metadata = { title: "Privacy — Bili Mate" };

export default function PrivacyPage() {
  return (
    <>
      <p className="eyebrow">Privacy</p>
      <h1>Data processing and retention</h1>
      <h2>What this service processes</h2>
      <p>
        An assessment contains clinical facts about a baby — gestation, elapsed ages, bilirubin
        values, clinical features and treatment state — with no name, NHS number, date of birth or
        other identifier. The interface cannot accept free text, so identifying details cannot be
        entered.
      </p>
      <h2>What stays in your browser</h2>
      <p>
        Birth, assessment and collection dates and times stay in this browser&rsquo;s memory only.
        The page derives elapsed ages locally and sends only elapsed minutes to the server.
        Everything you enter is lost when you clear the assessment, navigate away or close the tab
        — by design.
      </p>
      <h2>What the server keeps</h2>
      <p>
        Nothing clinical. The server evaluates the assessment in memory and discards the request
        and response after replying. Operational logs record only request identifiers, routes,
        status codes and timings, and expire after 30 days. Edge security systems process source IP
        addresses separately for abuse protection, for at most 30 days.
      </p>
      <h2>What this service will never do</h2>
      <p>
        No data is sold, used for advertising, used to train models, or combined to profile
        clinicians or patients. No advertising, session replay or social widgets run on assessment
        or result pages.
      </p>
      <p className="small muted">
        The professional-use confirmation stored for your session contains no clinical data and is
        not consent for patient-data processing.
      </p>
    </>
  );
}
