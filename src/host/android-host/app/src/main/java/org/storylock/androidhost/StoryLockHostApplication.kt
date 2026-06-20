package org.storylock.androidhost

import android.app.Application
import org.storylock.androidhost.host.AndroidHostConfig
import org.storylock.androidhost.host.AndroidHostServer
import org.storylock.androidhost.host.AndroidQuestionSetRepository
import org.storylock.androidhost.host.HostBindingStore
import org.storylock.androidhost.host.HostGatewayClient
import org.storylock.androidhost.host.HostRegistrationManager
import org.storylock.androidhost.host.LocalAuthorizationRuntime
import org.storylock.androidhost.host.StoryLockAndroidHostService
import org.storylock.androidhost.security.AndroidKeystoreSecretStore
import org.storylock.androidhost.security.AttachedActivityConfirmation
import org.storylock.androidhost.security.LocalUserConfirmation

class StoryLockHostApplication : Application() {
  lateinit var bindingStore: HostBindingStore
    private set

  lateinit var hostConfig: AndroidHostConfig
    private set

  lateinit var localConfirmation: LocalUserConfirmation
    private set

  lateinit var registrationManager: HostRegistrationManager
    private set

  lateinit var hostService: StoryLockAndroidHostService
    private set

  lateinit var server: AndroidHostServer
    private set

  override fun onCreate() {
    super.onCreate()

    bindingStore = HostBindingStore(this)
    val bootstrapConfig = AndroidHostConfig(
      port = BuildConfig.STORYLOCK_HOST_PORT,
      identityId = BuildConfig.STORYLOCK_IDENTITY_ID,
      sharedSecret = BuildConfig.STORYLOCK_SHARED_SECRET,
      deviceId = "",
      appInstanceId = "",
      gatewayBaseUrl = BuildConfig.STORYLOCK_GATEWAY_URL,
      preferredConnectMode = BuildConfig.STORYLOCK_CONNECT_MODE,
    )
    val bootstrapState = bindingStore.current(bootstrapConfig)

    hostConfig = AndroidHostConfig(
      port = BuildConfig.STORYLOCK_HOST_PORT,
      identityId = BuildConfig.STORYLOCK_IDENTITY_ID,
      sharedSecret = BuildConfig.STORYLOCK_SHARED_SECRET,
      deviceId = bootstrapState.deviceId,
      appInstanceId = bootstrapState.appInstanceId,
      gatewayBaseUrl = BuildConfig.STORYLOCK_GATEWAY_URL,
      preferredConnectMode = BuildConfig.STORYLOCK_CONNECT_MODE,
    )

    val secretStore = AndroidKeystoreSecretStore(this)
    val questionSet = AndroidQuestionSetRepository(this).loadActiveQuestionSet()
    require(questionSet.identityId == hostConfig.identityId) {
      "Android question set identityId (${questionSet.identityId}) does not match host identityId (${hostConfig.identityId})"
    }
    val runtime = LocalAuthorizationRuntime(
      identityId = questionSet.identityId,
      questionSetVersion = questionSet.questionSetVersion,
      normalizationVersion = questionSet.normalizationVersion,
      questions = questionSet.questions,
    )
    localConfirmation = AttachedActivityConfirmation()
    hostService = StoryLockAndroidHostService(
      config = hostConfig,
      secretStore = secretStore,
      localConfirmation = localConfirmation,
      runtime = runtime,
      connectivityProvider = {
        registrationManager.healthJson()
      },
    )
    registrationManager = HostRegistrationManager(
      config = hostConfig,
      bindingStore = bindingStore,
      gatewayClient = HostGatewayClient(hostConfig),
      hostService = hostService,
    )
    server = AndroidHostServer(
      config = hostConfig,
      hostService = hostService,
    )
    server.start()
    registrationManager.ensureStarted()
  }
}
