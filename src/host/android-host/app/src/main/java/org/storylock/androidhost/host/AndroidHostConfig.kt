package org.storylock.androidhost.host

data class AndroidHostConfig(
  val port: Int,
  val identityId: String,
  val sharedSecret: String,
  val deviceId: String,
  val appInstanceId: String,
  val gatewayBaseUrl: String = "",
  val preferredConnectMode: String = "relay_url",
  val questionSetVersion: String = "android-v1",
  val activeQuestionCount: Int = 24,
)
