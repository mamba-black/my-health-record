Feature: User Sign Up

  Scenario: Sign up succeeds with valid UUID v7
    Given a running gRPC user service and database environment
    When a sign up request with a valid UUID v7 is sent
    Then the sign up response is successful and the user is persisted in the database

  Scenario: Sign up fails when user ID is not UUID v7
    Given a running gRPC user service and database environment
    When a sign up request with an invalid UUID v4 is sent
    Then the sign up response returns an error indicating invalid UUID v7
