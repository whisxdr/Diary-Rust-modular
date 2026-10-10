import { request } from '@/lib/api-client';

import type { Session } from './types';

export const authApi = {
  session: (signal?: AbortSignal) => request<Session>({ path: '/auth/session', signal }),

  login: (body: { email: string; password: string }) =>
    request<Session>({ path: '/auth/login', method: 'POST', body }),

  register: (body: { email: string; display_name: string; password: string }) =>
    request<Session>({ path: '/auth/register', method: 'POST', body }),

  logout: () => request<{ message: string }>({ path: '/auth/logout', method: 'POST' }),

  changePassword: (body: { current_password: string; new_password: string }) =>
    request<{ message: string }>({ path: '/auth/password', method: 'POST', body }),
};