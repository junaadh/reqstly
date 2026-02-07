import { Link, useNavigate, Outlet } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';

export function AppLayout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="header-glass sticky top-0 z-50 shadow-[0_10px_30px_rgba(15,23,42,0.08)]">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center gap-2 group">
              <div className="w-8 h-8 bg-gradient-to-br from-teal-500 to-sky-500 rounded-xl flex items-center justify-center shadow-[0_10px_24px_rgba(14,116,144,0.3)] group-hover:shadow-[0_16px_30px_rgba(14,116,144,0.36)] transition-all">
                <span className="text-white font-bold text-sm">R</span>
              </div>
              <h1 className="text-xl font-display font-semibold text-slate-900">
                Reqstly
              </h1>
            </Link>

            {/* Navigation */}
            <nav className="flex items-center gap-1">
              <Link
                to="/"
                className="text-slate-600 hover:text-slate-900 hover:bg-white/70 px-4 py-2 rounded-full text-sm font-medium transition-all"
              >
                Dashboard
              </Link>
              <Link
                to="/requests"
                className="text-slate-600 hover:text-slate-900 hover:bg-white/70 px-4 py-2 rounded-full text-sm font-medium transition-all"
              >
                My Requests
              </Link>
              <Link
                to="/requests/new"
                className="btn-primary text-sm"
              >
                New Request
              </Link>

              {/* User menu */}
              <div className="flex items-center gap-3 ml-4 pl-4 border-l soft-divider">
                <Link
                  to="/profile"
                  className="flex items-center gap-2 text-slate-700 hover:text-slate-900 text-sm font-medium transition-all hover:bg-white/70 px-3 py-2 rounded-full"
                >
                  <div className="w-7 h-7 bg-gradient-to-br from-teal-400 to-sky-500 rounded-full flex items-center justify-center text-white text-xs font-semibold shadow-md">
                    {user?.name?.charAt(0).toUpperCase()}
                  </div>
                  {user?.name}
                </Link>
                <button
                  onClick={handleLogout}
                  className="text-slate-500 hover:text-slate-800 text-sm font-medium transition-colors"
                >
                  Logout
                </button>
              </div>
            </nav>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Outlet />
      </main>
    </div>
  );
}
