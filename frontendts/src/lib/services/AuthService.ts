import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import {page} from "$app/state";
import type Page from "$lib/domain/Page";
import {log} from "$lib/services/LoggerService";
import {goto} from "$app/navigation";
import {base} from "$app/paths";

interface AuthState {
    isAuthenticated: boolean;
    token: string | null;
    user: any | null; // Puedes definir una interfaz más detallada para tu usuario
}

const initialAuth: AuthState = {
    isAuthenticated: false,
    token: null,
    user: null,
};

// Intenta cargar el estado desde localStorage al inicio
if (browser) {
    const storedToken = localStorage.getItem('auth_token');
    if (storedToken) {
        initialAuth.isAuthenticated = true;
        initialAuth.token = storedToken;
        // Opcional: decodificar el token para obtener info del usuario si es un JWT
        // initialAuth.user = parseJwt(storedToken);
    }
}

export const authStore = writable<AuthState>(initialAuth);

// Funciones para actualizar el store
export const setAuthToken = (token: string, user: any = null) => {
    if (browser) {
        localStorage.setItem('auth_token', token);
    }
    authStore.set({ isAuthenticated: true, token, user });

    const previousPage = page.state as Page;
    log.debug(`AuthService: previus path ${previousPage.path}`);

    let url = previousPage.path ? `${base}${previousPage.path}` : `${base}/`;

    log.debug(`AuthService: redirecting to ${url}`);
    goto(url);
};

export const clearAuth = () => {
    if (browser) {
        localStorage.removeItem('auth_token');
    }
    authStore.set({ isAuthenticated: false, token: null, user: null });
};

// Función de ejemplo para parsear un JWT (para mostrar info del usuario si lo necesitas)
// function parseJwt(token: string) {
//     try {
//         const base64Url = token.split('.')[1];
//         const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
//         const jsonPayload = decodeURIComponent(atob(base64).split('').map(function(c) {
//             return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
//         }).join(''));
//         return JSON.parse(jsonPayload);
//     } catch (e) {
//         console.error('Error parsing JWT:', e);
//         return null;
//     }
// }
