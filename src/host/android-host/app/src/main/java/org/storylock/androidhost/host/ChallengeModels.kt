package org.storylock.androidhost.host

data class ChallengeCell(
  val cellId: String,
  val questionId: String,
  val promptText: String,
  val answer: String,
)

data class AuthorizationSession(
  val authorizationId: String,
  val allowedAction: String,
  val objectRef: String,
)
