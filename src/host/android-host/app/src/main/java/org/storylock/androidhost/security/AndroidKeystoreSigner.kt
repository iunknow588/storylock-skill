package org.storylock.androidhost.security

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyPair
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.Signature
import java.security.interfaces.ECPublicKey
import java.security.spec.ECGenParameterSpec
import org.json.JSONObject

class AndroidKeystoreSigner {
  private val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }

  fun getOrCreateSigningKey(alias: String, keyId: String, requireUserAuthentication: Boolean = false): JSONObject {
    val metadata = readMetadata(alias, keyId)
    if (metadata != null) {
      return metadata
    }
    val keyPair = getOrCreateKeyPair(alias, requireUserAuthentication)
    return buildMetadata(alias, keyId, keyPair.public.encoded, requireUserAuthentication)
  }

  fun sign(alias: String, payload: ByteArray): String {
    val privateKey = keyStore.getKey(alias, null) as? PrivateKey
      ?: getOrCreateKeyPair(alias, requireUserAuthentication = false).private
    val signature = Signature.getInstance(SIGNATURE_ALGORITHM)
    signature.initSign(privateKey)
    signature.update(payload)
    return Base64.encodeToString(signature.sign(), Base64.NO_WRAP)
  }

  fun delete(alias: String) {
    if (keyStore.containsAlias(alias)) {
      keyStore.deleteEntry(alias)
    }
  }

  private fun readMetadata(alias: String, keyId: String): JSONObject? {
    val certificate = keyStore.getCertificate(alias) ?: return null
    val publicKey = certificate.publicKey as? ECPublicKey ?: return null
    return buildMetadata(alias, keyId, publicKey.encoded, requireUserAuthentication = false)
  }

  private fun buildMetadata(
    alias: String,
    keyId: String,
    publicKey: ByteArray,
    requireUserAuthentication: Boolean,
  ): JSONObject {
    return JSONObject()
      .put("alias", alias)
      .put("keyId", keyId)
      .put("algorithm", RESULT_ALGORITHM)
      .put("signatureFormat", "base64-der")
      .put("curve", "secp256r1")
      .put("publicKeySpki", Base64.encodeToString(publicKey, Base64.NO_WRAP))
      .put("storage", "android_keystore_asymmetric_signer")
      .put("userAuthenticationRequired", requireUserAuthentication)
  }

  private fun getOrCreateKeyPair(alias: String, requireUserAuthentication: Boolean): KeyPair {
    val existingPublicKey = keyStore.getCertificate(alias)?.publicKey
    val existingPrivateKey = keyStore.getKey(alias, null) as? PrivateKey
    if (existingPublicKey != null && existingPrivateKey != null) {
      return KeyPair(existingPublicKey, existingPrivateKey)
    }
    val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC, ANDROID_KEYSTORE)
    val spec = KeyGenParameterSpec.Builder(
      alias,
      KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY,
    )
      .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
      .setDigests(KeyProperties.DIGEST_SHA256, KeyProperties.DIGEST_SHA512)
      .setUserAuthenticationRequired(requireUserAuthentication)
      .build()
    generator.initialize(spec)
    return generator.generateKeyPair()
  }

  private companion object {
    const val ANDROID_KEYSTORE = "AndroidKeyStore"
    const val SIGNATURE_ALGORITHM = "SHA256withECDSA"
    const val RESULT_ALGORITHM = "android-keystore-ecdsa-p256-sha256"
  }
}
