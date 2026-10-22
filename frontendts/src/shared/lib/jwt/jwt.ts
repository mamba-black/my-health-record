export interface GoogleJwtClaims {
  sub?: string;
  email?: string;
  given_name?: string;
  family_name?: string;
  name?: string;
  picture?: string;
  exp?: number;
}

/**
 * Decodifica el payload de un JWT (`id_token` de Google) en formato Base64URL.
 */
export function decodeGoogleJwt(idToken: string): GoogleJwtClaims | null {
  try {
    const segments = idToken.trim().split(".");
    if (segments.length !== 3 || !segments[1]) {
      return null;
    }

    const base64Url = segments[1];
    const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");
    const padded = base64.padEnd(
      base64.length + ((4 - (base64.length % 4)) % 4),
      "=",
    );

    const binaryString = atob(padded);
    const bytes = Uint8Array.from(binaryString, (char) => char.charCodeAt(0));
    const jsonPayload = new TextDecoder("utf-8").decode(bytes);

    return JSON.parse(jsonPayload) as GoogleJwtClaims;
  } catch {
    return null;
  }
}

/**
 * Genera un identificador UUID v7 (RFC 9562) ordenado temporalmente.
 */
export function generateUuidV7(): string {
  const randomBytes = new Uint8Array(16);
  crypto.getRandomValues(randomBytes);

  const timestampMs = BigInt(Date.now());
  randomBytes[0] = Number((timestampMs >> 40n) & 0xffn);
  randomBytes[1] = Number((timestampMs >> 32n) & 0xffn);
  randomBytes[2] = Number((timestampMs >> 24n) & 0xffn);
  randomBytes[3] = Number((timestampMs >> 16n) & 0xffn);
  randomBytes[4] = Number((timestampMs >> 8n) & 0xffn);
  randomBytes[5] = Number(timestampMs & 0xffn);

  // Version 7 (0111)
  randomBytes[6] = (randomBytes[6] & 0x0f) | 0x70;
  // Variant RFC 9562 (10xx)
  randomBytes[8] = (randomBytes[8] & 0x3f) | 0x80;

  const hex = Array.from(randomBytes, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");

  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}
