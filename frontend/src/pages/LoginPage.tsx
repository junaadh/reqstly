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
      <div className="flex border-b soft-divider">
        <button
          onClick={() => setIsLogin(true)}
          className={`flex-1 py-3 text-sm font-semibold transition-all ${
            isLogin
              ? 'text-teal-700 border-b-2 border-teal-500'
              : 'text-slate-500 hover:text-slate-700'
          }`}
        >
          Sign In
        </button>
        <button
          onClick={() => setIsLogin(false)}
          className={`flex-1 py-3 text-sm font-semibold transition-all ${
            !isLogin
              ? 'text-teal-700 border-b-2 border-teal-500'
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
          className="btn-secondary w-full flex justify-center items-center gap-3 text-sm"
        >
          Continue with Azure AD
        </button>

        <button
          type="button"
          onClick={handlePasskeyLogin}
          className="btn-secondary w-full flex justify-center items-center gap-3 text-sm"
        >
          Continue with Passkey
        </button>
      </div>

      {/* Divider */}
      <div className="relative my-6">
        <div className="absolute inset-0 flex items-center">
          <div className="w-full border-t soft-divider"></div>
        </div>
        <div className="relative flex justify-center text-sm">
          <span className="px-4 bg-white/80 text-slate-500 font-medium">Or continue with email</span>
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
              className="input-field w-full"
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
              className="input-field w-full"
              placeholder="••••••••"
            />
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="btn-primary w-full disabled:opacity-50 disabled:cursor-not-allowed flex justify-center items-center gap-2"
          >
            {isLoading ? 'Signing in...' : 'Sign In'}
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
              className="input-field w-full"
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
              className="input-field w-full"
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
              className="input-field w-full"
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
              className="input-field w-full"
              placeholder="••••••••"
            />
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="btn-primary w-full disabled:opacity-50 disabled:cursor-not-allowed flex justify-center items-center gap-2"
          >
            {isLoading ? 'Creating account...' : 'Create Account'}
          </button>
        </form>
      )}
    </div>
  );
}
