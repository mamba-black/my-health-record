import { resolve } from "$app/paths";

export const ROOT = resolve(`/`);
export const HISTORY = resolve(`/patients`);
export const LOGIN = resolve(`/login`);
export const SIGNUP = resolve(`/signup`);
export const COMPLETE_PROFILE = resolve(`/complete-profile`);
export const CALLBACK = resolve(`/auth/google/callback`);
export const SEARCH = resolve(`/search`);
export const MY_CLINICS = resolve(`/my-clinics`);
export const ACCESS_DENIED = resolve(`/access-denied`);

export class Page {
  constructor(public path: string) {}
}

export default Page;
