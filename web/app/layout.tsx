import type { Metadata } from "next";
import { IBM_Plex_Mono, IBM_Plex_Sans } from "next/font/google";
import Link from "next/link";

import "./globals.css";

const plexSans = IBM_Plex_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--plex-sans",
});

const plexMono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: ["400", "600"],
  variable: "--plex-mono",
});

export const metadata: Metadata = {
  title: "Bili Mate — neonatal jaundice decision support",
  description:
    "Clinical decision support for registered UK healthcare professionals assessing jaundice in newborn babies, based on NICE guideline CG98.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en-GB" className={`${plexSans.variable} ${plexMono.variable}`}>
      <body>
        <a className="skip-link" href="#main">
          Skip to main content
        </a>
        <div className="professional-banner" role="note">
          <div className="shell">
            For registered UK healthcare professionals only. This service has no login and does not
            verify clinician identity. Output is advisory and never replaces clinical judgement.
          </div>
        </div>
        <header className="site-header">
          <div className="shell">
            <Link href="/" className="wordmark">
              Bili Mate
            </Link>
            <span className="wordmark-sub">Neonatal jaundice decision support — NICE CG98</span>
            <nav className="site-nav" aria-label="Site">
              <Link href="/assessment">Assessment</Link>
              <Link href="/about">About</Link>
              <Link href="/service-status">Service status</Link>
            </nav>
          </div>
        </header>
        <main id="main" className="shell">
          {children}
        </main>
        <footer className="site-footer">
          <div className="shell">
            <p>
              This product includes content from NICE guideline CG98 (Jaundice in newborn babies
              under 28 days), used under the NICE UK Open Content Licence. NICE guidance is prepared
              for the National Health Service in England and may be updated or withdrawn. NICE has
              not endorsed, and is not responsible for, Bili Mate.
            </p>
            <p>
              <Link href="/about">About</Link> · <Link href="/privacy">Privacy</Link> ·{" "}
              <Link href="/accessibility">Accessibility</Link> ·{" "}
              <Link href="/service-status">Service status</Link>
            </p>
          </div>
        </footer>
      </body>
    </html>
  );
}
