export const metadata = { title: "Accessibility — Bili Mate" };

export default function AccessibilityPage() {
  return (
    <>
      <p className="eyebrow">Accessibility</p>
      <h1>Accessibility statement</h1>
      <p>
        Bili Mate is designed to meet WCAG 2.2 level AA. The interface works with keyboard-only
        operation and screen readers, remains usable at 200% zoom, and never communicates clinical
        state through colour alone: emergency and immediate findings use text, iconography, heading
        structure and inverted black-and-white banners, and chart lines are distinguished by dash
        pattern with a full table alternative.
      </p>
      <p>
        Motion is not used to convey meaning, and the interface respects your reduced-motion
        preference. Touch targets meet the minimum 24 by 24 pixel size, with larger targets for
        primary clinical actions.
      </p>
      <h2>Known limitations</h2>
      <p>
        This demonstration deployment has not yet completed its formal manual accessibility audit
        with representative screen-reader users; that review is a release gate before any clinical
        pilot.
      </p>
      <h2>Feedback</h2>
      <p>
        If you find an accessibility problem, report it through the service contact on the About
        page so it can be fixed before clinical use.
      </p>
    </>
  );
}
