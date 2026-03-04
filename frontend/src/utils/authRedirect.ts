const AUTH_PAGE_PATHS = new Set(['/login', '/signup', '/register', '/invite']);

export const DEFAULT_POST_LOGIN_PATH = '/plants';

export function isSafeRedirectPath(path: string): boolean {
  if (!path || !path.startsWith('/')) {
    return false;
  }

  // Prevent protocol-relative redirects (e.g. //example.com).
  if (path.startsWith('//')) {
    return false;
  }

  const [pathname] = path.split(/[?#]/);
  return !AUTH_PAGE_PATHS.has(pathname);
}

export function buildLoginRedirectPath(pathname: string, search = '', hash = ''): string {
  const attemptedPath = `${pathname}${search}${hash}`;
  if (!isSafeRedirectPath(attemptedPath)) {
    return '/login';
  }

  const redirectParam = encodeURIComponent(attemptedPath);
  return `/login?redirect=${redirectParam}`;
}

export function resolvePostLoginPath(
  search: string,
  fallback = DEFAULT_POST_LOGIN_PATH
): string {
  const params = new URLSearchParams(search);
  const redirect = params.get('redirect');
  if (redirect && isSafeRedirectPath(redirect)) {
    return redirect;
  }

  return fallback;
}
