package org.storylock.androidhost.host

import android.content.Context
import android.net.Uri
import java.util.UUID

data class HostBindingState(
  val deviceId: String,
  val appInstanceId: String,
  val identityId: String,
  val gatewayBaseUrl: String,
  val preferredMode: String,
  val bindingToken: String?,
  val registrationId: String?,
  val relayUrl: String?,
  val relayPollUrl: String?,
  val relayRespondUrl: String?,
  val deepLink: String?,
  val lastRegisteredAt: Long?,
  val lastSeenAt: Long?,
  val lastError: String?,
) {
  fun hasGateway(): Boolean = gatewayBaseUrl.isNotBlank()
  fun usesRelay(): Boolean = preferredMode == "relay_url" || preferredMode == "deep_link"
  fun isRegistered(): Boolean = registrationId != null
  fun isOnline(now: Long = System.currentTimeMillis(), onlineTtlMs: Long = 120_000): Boolean {
    val seenAt = lastSeenAt ?: return false
    return lastError == null && now - seenAt <= onlineTtlMs
  }

  fun displayStatus(now: Long = System.currentTimeMillis()): String {
    if (!hasGateway()) {
      return "unbound"
    }
    if (!isRegistered()) {
      return "bound_not_registered"
    }
    return if (isOnline(now)) "online" else "offline"
  }
}

class HostBindingStore(
  context: Context,
) {
  private val preferences = context.getSharedPreferences("storylock_host_binding", Context.MODE_PRIVATE)

  fun current(config: AndroidHostConfig): HostBindingState {
    return HostBindingState(
      deviceId = ensureStableValue(KEY_DEVICE_ID),
      appInstanceId = ensureStableValue(KEY_APP_INSTANCE_ID),
      identityId = config.identityId,
      gatewayBaseUrl = nonBlankPreference(KEY_GATEWAY_BASE_URL) ?: config.gatewayBaseUrl,
      preferredMode = nonBlankPreference(KEY_PREFERRED_MODE) ?: config.preferredConnectMode,
      bindingToken = preferences.getString(KEY_BINDING_TOKEN, null),
      registrationId = preferences.getString(KEY_REGISTRATION_ID, null),
      relayUrl = preferences.getString(KEY_RELAY_URL, null),
      relayPollUrl = preferences.getString(KEY_RELAY_POLL_URL, null),
      relayRespondUrl = preferences.getString(KEY_RELAY_RESPOND_URL, null),
      deepLink = preferences.getString(KEY_DEEP_LINK, null),
      lastRegisteredAt = getLong(KEY_LAST_REGISTERED_AT),
      lastSeenAt = getLong(KEY_LAST_SEEN_AT),
      lastError = preferences.getString(KEY_LAST_ERROR, null),
    )
  }

  fun applyDeepLink(uri: Uri, config: AndroidHostConfig): HostBindingState {
    val current = current(config)
    val gatewayUrl = uri.getQueryParameter("gateway_url")?.trim().orEmpty()
    val preferredMode = uri.getQueryParameter("preferred_mode")?.trim().orEmpty()
    val bindingToken = uri.getQueryParameter("binding_token")?.trim().orEmpty()
    preferences.edit()
      .putString(KEY_GATEWAY_BASE_URL, if (gatewayUrl.isBlank()) current.gatewayBaseUrl else gatewayUrl)
      .putString(KEY_PREFERRED_MODE, if (preferredMode.isBlank()) current.preferredMode else preferredMode)
      .putString(KEY_BINDING_TOKEN, if (bindingToken.isBlank()) current.bindingToken else bindingToken)
      .putString(KEY_DEEP_LINK, uri.toString())
      .apply()
    return current(config)
  }

  fun updateRegistration(
    config: AndroidHostConfig,
    registrationId: String?,
    relayUrl: String?,
    relayPollUrl: String?,
    relayRespondUrl: String?,
    deepLink: String?,
  ): HostBindingState {
    val now = System.currentTimeMillis()
    preferences.edit()
      .putString(KEY_REGISTRATION_ID, registrationId)
      .putString(KEY_RELAY_URL, relayUrl)
      .putString(KEY_RELAY_POLL_URL, relayPollUrl)
      .putString(KEY_RELAY_RESPOND_URL, relayRespondUrl)
      .putString(KEY_DEEP_LINK, deepLink ?: preferences.getString(KEY_DEEP_LINK, null))
      .putLong(KEY_LAST_REGISTERED_AT, now)
      .putLong(KEY_LAST_SEEN_AT, now)
      .putString(KEY_LAST_ERROR, null)
      .apply()
    return current(config)
  }

  fun markSeen(config: AndroidHostConfig): HostBindingState {
    preferences.edit()
      .putLong(KEY_LAST_SEEN_AT, System.currentTimeMillis())
      .putString(KEY_LAST_ERROR, null)
      .apply()
    return current(config)
  }

  fun markError(config: AndroidHostConfig, message: String): HostBindingState {
    preferences.edit()
      .putString(KEY_LAST_ERROR, message)
      .apply()
    return current(config)
  }

  private fun ensureStableValue(key: String): String {
    val existing = preferences.getString(key, null)
    if (!existing.isNullOrBlank()) {
      return existing
    }
    val generated = UUID.randomUUID().toString()
    preferences.edit().putString(key, generated).apply()
    return generated
  }

  private fun nonBlankPreference(key: String): String? {
    val value = preferences.getString(key, null)?.trim()
    return value?.ifBlank { null }
  }

  private fun getLong(key: String): Long? {
    return if (preferences.contains(key)) preferences.getLong(key, 0L) else null
  }

  private companion object {
    const val KEY_DEVICE_ID = "device_id"
    const val KEY_APP_INSTANCE_ID = "app_instance_id"
    const val KEY_GATEWAY_BASE_URL = "gateway_base_url"
    const val KEY_PREFERRED_MODE = "preferred_mode"
    const val KEY_BINDING_TOKEN = "binding_token"
    const val KEY_REGISTRATION_ID = "registration_id"
    const val KEY_RELAY_URL = "relay_url"
    const val KEY_RELAY_POLL_URL = "relay_poll_url"
    const val KEY_RELAY_RESPOND_URL = "relay_respond_url"
    const val KEY_DEEP_LINK = "deep_link"
    const val KEY_LAST_REGISTERED_AT = "last_registered_at"
    const val KEY_LAST_SEEN_AT = "last_seen_at"
    const val KEY_LAST_ERROR = "last_error"
  }
}
