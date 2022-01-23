import org.bouncycastle.jce.provider.BouncyCastleProvider
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should
import scribe.{debug, Level, Logger}
import sun.security.rsa.RSAPublicKeyImpl

import java.security.{KeyFactory, Security}
import java.security.spec.X509EncodedKeySpec
import java.util.Base64
import javax.crypto.Cipher

class RSATest extends AnyFlatSpec with should.Matchers {
  Logger.root.clearHandlers().clearModifiers().withHandler(minimumLevel = Some(Level.Info)).replace()

  val publicKeyContent =
    """
      |-----BEGIN PUBLIC KEY-----
      |MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEApglWWF0IgxIRgT2n+6UP
      |qM3vHntsWYgEWofO7BMClmfgcIMTqwBCWX+iu2t5OBq6D5p2uzVpvLZiZvFPDJ/p
      |Asa9RdIdmqYIjzTkAtKKLcefuRT1TOHugLIZOnpIxw94fEz4bFyNQM8r/jAG5BS6
      |BQHIAQNxRloxlu3CkgkMLBtUIW5gfVL+/oEih+hRQLkbYlg1mXz6B1r+u9WABYbc
      |iMw5ib5KVH2jReeQlKOvKWRRqgcv4s3hZg2Guonn5cRw1wyCpiKkB05QcRCh4GBW
      |Qj2Nn0ToWy9ehQ2Gd+11YaJ2JKWtTvKeD8g1EPc0xLKBYL5gkjlG7v/Ze4TZSIeV
      |cwIDAQAB
      |-----END PUBLIC KEY-----
      |""".stripMargin

  "Lectura de key RSA" should "" in {
    val publicKeyContentClean = publicKeyContent
      .replace("-----BEGIN PUBLIC KEY-----", "")
      .replace("-----END PUBLIC KEY-----", "")
      .replace("\n", "")
    val msg = "Hola mundo, probando el mundo scala"

    Security.addProvider(new BouncyCastleProvider)

    val publicKeyDecoded = Base64.getDecoder.decode(publicKeyContentClean)
    info(s"publicKeyContentClean: $publicKeyDecoded")
    val x509EncodedKeySpec = new X509EncodedKeySpec(publicKeyDecoded)

    {
      val keyFactory = KeyFactory.getInstance("RSA")
      val publicKey = keyFactory.generatePublic(x509EncodedKeySpec).asInstanceOf[RSAPublicKeyImpl]

      debug(s"publicKey: $publicKey")
      debug(s"publicKey.getModulus: ${publicKey.getModulus}")
      debug(s"publicKey.getModulus.bitLength: ${publicKey.getModulus.bitLength()}")
      info(s"publicKey.getPublicExponent: ${publicKey.getPublicExponent}")
      info(s"publicKey.getAlgorithm: ${publicKey.getAlgorithm}")
      info(s"publicKey.getAlgorithmId: ${publicKey.getAlgorithmId}")
      info(s"publicKey.getFormat: ${publicKey.getFormat}")
      info(s"publicKey.getEncoded: ${publicKey.getEncoded}")
      debug(s"publicKey.getParams: ${publicKey.getParams}")

      val rsa = Cipher.getInstance("RSA/ECB/PKCS1Padding")
      //val rsa = Cipher.getInstance("RSA/NONE/PKCS1Padding ")
      //val rsa = Cipher.getInstance("RSA/NONE/NoPadding")
      rsa.init(Cipher.ENCRYPT_MODE, publicKey)
      for (i <- 1 to 20) {
        info(s"msgEncrypt: ${new String(Base64.getEncoder.encode(rsa.doFinal(msg.getBytes())))}")
      }
    }

  }

}
