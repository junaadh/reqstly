import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { authApi, ApiError } from '../api/client';
import { toast } from 'sonner';

export function LoginPage() {
  const { refresh } = useAuth();
  const navigate = useNavigate();
  const [isLogin, setIsLogin] = useState(true);
  const [isLoading, setIsLoading] = useState(false);

  // Login form state
  const [loginEmail, setLoginEmail] = useState('');
  const [loginPassword, setLoginPassword] = useState('');

  // Signup form state
  const [signupName, setSignupName] = useState('');
  const [signupEmail, setSignupEmail] = useState('');
  const [signupPassword, setSignupPassword] = useState('');
  const [signupConfirmPassword, setSignupConfirmPassword] = useState('');

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      await authApi.passwordLogin(loginEmail, loginPassword);
      toast.success('Signed in successfully');
      await refresh();
      navigate('/');
    } catch (err) {
      const errorMessage = err instanceof ApiError ? (err.message || 'Login failed') : 'Login failed';
      toast.error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignup = async (e: React.FormEvent) => {
    e.preventDefault();

    if (signupPassword !== signupConfirmPassword) {
      toast.error('Passwords do not match');
      return;
    }

    if (signupPassword.length < 8) {
      toast.error('Password must be at least 8 characters');
      return;
    }

    setIsLoading(true);

    try {
      await authApi.passwordSignup(signupEmail, signupName, signupPassword);
      toast.success('Account created successfully');
      await refresh();
      navigate('/');
    } catch (err) {
      const errorMessage = err instanceof ApiError ? (err.message || 'Signup failed') : 'Signup failed';
      toast.error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleAzureLogin = () => {
    toast.info('AzureAD authentication', {
      description: 'AzureAD authentication is not configured yet. Please use email/password.',
    });
  };

  const handlePasskeyLogin = () => {
    toast.info('Passkey authentication', {
      description: 'Passkey authentication is not configured yet. Please use email/password.',
    });
  };

  return (
    <div className="space-y-6">
      {/* Tabs */}
      <div className="flex border-b border-slate-200">
        <button
          onClick={() => setIsLogin(true)}
          className={`flex-1 py-3 text-sm font-semibold transition-all ${
            isLogin
              ? 'text-indigo-600 border-b-2 border-indigo-600'
              : 'text-slate-500 hover:text-slate-700'
          }`}
        >
          Sign In
        </button>
        <button
          onClick={() => setIsLogin(false)}
          className={`flex-1 py-3 text-sm font-semibold transition-all ${
            !isLogin
              ? 'text-indigo-600 border-b-2 border-indigo-600'
              : 'text-slate-500 hover:text-slate-700'
          }`}
        >
          Sign Up
        </button>
      </div>

      {/* Social auth buttons - shown on both tabs */}
      <div className="space-y-3">
        <button
          type="button"
          onClick={handleAzureLogin}
          className="w-full flex justify-center items-center gap-3 px-4 py-3 border-2 border-slate-200 rounded-xl text-sm font-semibold text-slate-700 hover:bg-slate-50 hover:border-slate-300 transition-all"
        >
          <svg className="w-5 h-5" viewBox="0 0 23 23" fill="none">
            <path d="M11.5 0C5.15 0 0 5.15 0 11.5C0 17.85 5.15 23 11.5 23C17.85 23 23 17.85 23 11.5C23 5.15 17.85 0 11.5 0Z" fill="#F25022"/>
            <path d="M11.5 0C17.85 0 23 5.15 23 11.5C23 17.85 17.85 23 11.5 23C5.15 23 0 17.85 0 11.5C0 5.15 5.15 0 11.5 0Z" fill="#7FBA00" transform="translate(11.5, 0)"/>
            <path d="M11.5 0C17.85 0 23 5.15 23 11.5C23 17.85 17.85 23 11.5 23C5.15 23 0 17.85 0 11.5C0 5.15 5.15 0 11.5 0Z" fill="#00A4EF" transform="translate(0, 11.5)"/>
            <path d="M11.5 0C17.85 0 23 5.15 23 11.5C23 17.85 17.85 23 11.5 23C5.15 23 0 17.85 0 11.5C0 5.15 5.15 0 11.5 0Z" fill="#FFB900" transform="translate(11.5, 11.5)"/>
          </svg>
          Continue with Azure AD
        </button>

        <button
          type="button"
          onClick={handlePasskeyLogin}
          className="w-full flex justify-center items-center gap-3 px-4 py-3 border-2 border-slate-200 rounded-xl text-sm font-semibold text-slate-700 hover:bg-slate-50 hover:border-slate-300 transition-all"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
          </svg>
          Continue with Passkey
        </button>
      </div>

      {/* Divider */}
      <div className="relative my-6">
        <div className="absolute inset-0 flex items-center">
          <div className="w-full border-t border-slate-200"></div>
        </div>
        <div className="relative flex justify-center text-sm">
          <span className="px-4 bg-white text-slate-500 font-medium">Or continue with email</span>
        </div>
      </div>

      {isLogin ? (
        /* Login Form */
        <form onSubmit={handleLogin} className="space-y-4">
          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Email
            </label>
            <input
              type="email"
              required
              value={loginEmail}
              onChange={(e) => setLoginEmail(e.target.value)}
              className="w-full border-2 border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
              placeholder="you@example.com"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Password
            </label>
            <input
              type="password"
              required
              value={loginPassword}
              onChange={(e) => setLoginPassword(e.target.value)}
              className="w-full border-2 border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
              placeholder="••••••••"
            />
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white py-3 rounded-xl font-semibold shadow-lg shadow-indigo-500/20 hover:shadow-xl hover:shadow-indigo-500/30 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex justify-center items-center gap-2"
          >
            {isLoading ? (
              <>
                <svg className="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Signing in...
              </>
            ) : (
              'Sign In'
            )}
          </button>
        </form>
      ) : (
        /* Signup Form */
        <form onSubmit={handleSignup} className="space-y-4">
          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Full Name
            </label>
            <input
              type="text"
              required
              value={signupName}
              onChange={(e) => setSignupName(e.target.value)}
              className="w-full border-2 border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
              placeholder="John Doe"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Email
            </label>
            <input
              type="email"
              required
              value={signupEmail}
              onChange={(e) => setSignupEmail(e.target.value)}
              className="w-full border-2 border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
              placeholder="you@example.com"
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Password
            </label>
            <input
              type="password"
              required
              value={signupPassword}
              onChange={(e) => setSignupPassword(e.target.value)}
              className="w-full border-2 border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
              placeholder="••••••••"
            />
            {signupPassword.length < 8 && <p className="text-xs text-slate-500 mt-2 ml-1">Minimum 8 characters</p>}
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Confirm Password
            </label>
            <input
              type="password"
              required
              value={signupConfirmPassword}
              onChange={(e) => setSignupConfirmPassword(e.target.value)}
              className="w-full border-2 border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
              placeholder="••••••••"
            />
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white py-3 rounded-xl font-semibold shadow-lg shadow-indigo-500/20 hover:shadow-xl hover:shadow-indigo-500/30 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex justify-center items-center gap-2"
          >
            {isLoading ? (
              <>
                <svg className="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Creating account...
              </>
            ) : (
              'Create Account'
            )}
          </button>
        </form>
      )}
    </div>
  );
}
