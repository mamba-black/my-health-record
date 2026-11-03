import {
  type SignInResponse,
  type SignUpRequest,
  type SignUpResponse,
  userApi,
} from "$shared/api";

export function signInWithGoogle(idToken: string): Promise<SignInResponse> {
  return userApi.signIn({ idToken, providerId: "google.com" });
}

export function signUpAccount(request: SignUpRequest): Promise<SignUpResponse> {
  return userApi.signUp(request);
}
