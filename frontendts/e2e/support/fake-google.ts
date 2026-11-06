import { createSign, generateKeyPairSync, type KeyObject } from "node:crypto";
import { createServer } from "node:http";
import { FAKE_GOOGLE_PORT } from "./config";

// Hace de Google en las pruebas: publica una clave y firma tokens con ella. La clave
// `attacker` nunca se publica, así que sus firmas deben rechazarse.
const KEY_ID = "e2e-key";
const generateKey = () =>
  generateKeyPairSync("rsa", { modulusLength: 2048 });
const published = generateKey();
const attacker = generateKey();

const base64Url = (value: string | Buffer) =>
  Buffer.from(value).toString("base64url");

const sign = (claims: object, privateKey: KeyObject) => {
  const header = base64Url(
    JSON.stringify({ alg: "RS256", kid: KEY_ID, typ: "JWT" }),
  );
  const body = `${header}.${base64Url(JSON.stringify(claims))}`;
  const signature = createSign("RSA-SHA256").update(body).sign(privateKey);
  return `${body}.${base64Url(signature)}`;
};

createServer((request, response) => {
  if (request.method === "GET" && request.url === "/certs") {
    const jwk = published.publicKey.export({ format: "jwk" });
    response.setHeader("content-type", "application/json");
    response.end(
      JSON.stringify({
        keys: [{ ...jwk, kid: KEY_ID, alg: "RS256", use: "sig" }],
      }),
    );
    return;
  }

  if (request.method === "POST" && request.url === "/sign") {
    let payload = "";
    request.on("data", (chunk) => {
      payload += chunk;
    });
    request.on("end", () => {
      const { claims, signedByAttacker } = JSON.parse(payload);
      const key = signedByAttacker ? attacker : published;
      response.end(sign(claims, key.privateKey));
    });
    return;
  }

  response.statusCode = 404;
  response.end();
}).listen(FAKE_GOOGLE_PORT, "localhost");
