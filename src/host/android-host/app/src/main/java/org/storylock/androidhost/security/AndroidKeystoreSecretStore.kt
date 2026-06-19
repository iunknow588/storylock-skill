package org.storylock.androidhost.security

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import java.security.KeyStore

class AndroidKeystoreSecretStore(
  context: Context,
) : SecretStore {
  private val sharedPreferences = context.getSharedPreferences("storylock_keystore_store", Context.MODE_PRIVATE)
  private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

  override fun getSecret(alias: String): ByteArray? {
    val stored = sharedPreferences.getString(alias, null) ?: return null
    val parts = stored.split(':')
    require(parts.size == 2) { "invalid encrypted secret envelope" }
    val iv = Base64.decode(parts[0], Base64.NO_WRAP)
    val ciphertext = Base64.decode(parts[1], Base64.NO_WRAP)
    val key = getOrCreateKey(alias, requireUserAuthentication = false)
    val cipher = Cipher.getInstance(TRANSFORMATION)
    cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(GCM_TAG_BITS, iv))
    return cipher.doFinal(ciphertext)
  }

  override fun setSecret(alias: String, value: ByteArray, requireUserAuthentication: Boolean) {
    val key = getOrCreateKey(alias, requireUserAuthentication)
    val cipher = Cipher.getInstance(TRANSFORMATION)
    cipher.init(Cipher.ENCRYPT_MODE, key)
    val ciphertext = cipher.doFinal(value)
    val payload = "${Base64.encodeToString(cipher.iv, Base64.NO_WRAP)}:${Base64.encodeToString(ciphertext, Base64.NO_WRAP)}"
    sharedPreferences.edit().putString(alias, payload).apply()
  }

  override fun deleteSecret(alias: String) {
    sharedPreferences.edit().remove(alias).apply()
    if (keyStore.containsAlias(alias)) {
      keyStore.deleteEntry(alias)
    }
  }

  override fun listAliases(prefix: String?): List<String> {
    return sharedPreferences.all.keys
      .filter { prefix == null || it.startsWith(prefix) }
      .sorted()
  }

  private fun getOrCreateKey(alias: String, requireUserAuthentication: Boolean): SecretKey {
    val existing = keyStore.getKey(alias, null)
    if (existing is SecretKey) {
      return existing
    }
    val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
    val spec = KeyGenParameterSpec.Builder(
      alias,
      KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
    )
      .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
      .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
      .setUserAuthenticationRequired(requireUserAuthentication)
      .setRandomizedEncryptionRequired(true)
      .build()
    generator.init(spec)
    return generator.generateKey()
  }

  private companion object {
    const val TRANSFORMATION = "AES/GCM/NoPadding"
    const val GCM_TAG_BITS = 128
  }
}
