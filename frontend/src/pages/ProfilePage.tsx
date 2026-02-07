import { useAuth } from '../contexts/AuthContext';

export function ProfilePage() {
  const { user } = useAuth();

  if (!user) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
        Unable to load profile
      </div>
    );
  }

  return (
    <div className="max-w-2xl space-y-6">
      <div>
        <h1 className="text-2xl font-display font-semibold text-slate-900">Profile</h1>
        <p className="text-slate-600">Your account information</p>
      </div>

      <div className="surface-card">
        <div className="px-6 py-4 border-b soft-divider">
          <h2 className="text-lg font-semibold text-slate-900">Account Details</h2>
        </div>
        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-600 mb-1">
              Name
            </label>
            <div className="text-slate-900">{user.name}</div>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-600 mb-1">
              Email
            </label>
            <div className="text-slate-900">{user.email}</div>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-600 mb-1">
              Authentication Provider
            </label>
            <div className="text-slate-900 capitalize">{user.provider}</div>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-600 mb-1">
              Account Type
            </label>
            <div className="text-slate-900">
              {user.federated ? 'Federated (SSO)' : 'Local Account'}
            </div>
          </div>
        </div>
      </div>

      <div className="bg-sky-50/80 border border-sky-200/70 p-4 rounded-2xl">
        <p className="text-sm text-sky-800">
          Your account is managed through your organization's identity provider.
          To update your information, please contact your administrator.
        </p>
      </div>
    </div>
  );
}
