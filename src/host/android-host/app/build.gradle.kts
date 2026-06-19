plugins {
  id("com.android.application")
  id("org.jetbrains.kotlin.android")
}

android {
  namespace = "org.storylock.androidhost"
  compileSdk = 34
  buildToolsVersion = "35.0.0"

  defaultConfig {
    applicationId = "org.storylock.androidhost"
    minSdk = 26
    targetSdk = 34
    versionCode = 1
    versionName = "0.1.0"

    testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    buildConfigField("int", "STORYLOCK_HOST_PORT", "4500")
    buildConfigField("String", "STORYLOCK_IDENTITY_ID", "\"android-demo-001\"")
    buildConfigField("String", "STORYLOCK_SHARED_SECRET", "\"replace-with-strong-shared-secret\"")
    buildConfigField("String", "STORYLOCK_GATEWAY_URL", "\"https://yian.cdao.online\"")
    buildConfigField("String", "STORYLOCK_CONNECT_MODE", "\"relay_url\"")
  }

  buildTypes {
    release {
      isMinifyEnabled = false
      proguardFiles(
        getDefaultProguardFile("proguard-android-optimize.txt"),
        "proguard-rules.pro",
      )
    }
  }

  compileOptions {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
  }
  kotlinOptions {
    jvmTarget = "17"
  }
  buildFeatures {
    buildConfig = true
  }
}

dependencies {
  implementation("androidx.core:core-ktx:1.13.1")
  implementation("androidx.appcompat:appcompat:1.7.0")
  implementation("com.google.android.material:material:1.12.0")
  implementation("androidx.activity:activity-ktx:1.9.0")
  implementation("androidx.biometric:biometric:1.1.0")
  implementation("org.nanohttpd:nanohttpd:2.3.1")
  implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
}
