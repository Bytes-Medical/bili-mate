// Professional-use acknowledgement (WEB-001–WEB-003): stored in
// sessionStorage only, expires with the browser session, contains no
// clinical data, and is not consent for patient-data processing.

const KEY = "bili-mate.professional-acknowledgement";

export function hasAcknowledged(): boolean {
  try {
    return sessionStorage.getItem(KEY) === "acknowledged";
  } catch {
    return false;
  }
}

export function recordAcknowledgement(): void {
  try {
    sessionStorage.setItem(KEY, "acknowledged");
  } catch {
    // Storage unavailable: the gate simply reappears on navigation.
  }
}
