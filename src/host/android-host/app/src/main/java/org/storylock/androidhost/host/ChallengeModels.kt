package org.storylock.androidhost.host

data class ChallengeCell(
  val cellId: String,
  val questionId: String,
  val promptText: String,
  val answer: String,
  val position: Int,
)

data class ChallengeSession(
  val challengeId: String,
  val requiredStrength: String,
  val requiredCells: Int,
  val requiredThreshold: Int,
  val questionSetVersion: String,
  val failureCount: Int,
  val maxFailureCount: Int,
  val lockUntil: Long,
  val cells: List<ChallengeCell>,
)

data class ChallengeVerificationResult(
  val approved: Boolean,
  val matchedCount: Int,
  val requiredThreshold: Int,
  val failureCount: Int,
  val maxFailureCount: Int,
  val lockUntil: Long,
)

data class AuthorizationSession(
  val authorizationId: String,
  val allowedAction: String,
  val objectRef: String,
  val createdAt: Long,
  val expiresAt: Long,
  val status: String = "active",
)
