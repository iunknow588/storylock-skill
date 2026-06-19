package org.storylock.androidhost.security

interface SecretStore {
  fun getSecret(alias: String): ByteArray?
  fun setSecret(alias: String, value: ByteArray, requireUserAuthentication: Boolean = false)
  fun deleteSecret(alias: String)
  fun listAliases(prefix: String? = null): List<String>
}
