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
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50">
      {/* Header */}
      <header className="bg-white/80 backdrop-blur-lg border-b border-slate-200/50 sticky top-0 z-50 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center gap-2 group">
              <div className="w-8 h-8 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-lg flex items-center justify-center shadow-lg shadow-indigo-500/20 group-hover:shadow-xl group-hover:shadow-indigo-500/30 transition-all">
                <span className="text-white font-bold text-sm">R</span>
              </div>
              <h1 className="text-xl font-bold bg-gradient-to-r from-slate-900 to-slate-700 bg-clip-text text-transparent">
                Reqstly
              </h1>
            </Link>

            {/* Navigation */}
            <nav className="flex items-center gap-1">
              <Link
                to="/"
                className="text-slate-600 hover:text-slate-900 hover:bg-slate-100/80 px-4 py-2 rounded-lg text-sm font-medium transition-all"
              >
                Dashboard
              </Link>
              <Link
                to="/requests"
                className="text-slate-600 hover:text-slate-900 hover:bg-slate-100/80 px-4 py-2 rounded-lg text-sm font-medium transition-all"
              >
                My Requests
              </Link>
              <Link
                to="/requests/new"
                className="bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white px-4 py-2 rounded-lg text-sm font-medium shadow-lg shadow-indigo-500/20 hover:shadow-xl hover:shadow-indigo-500/30 transition-all"
              >
                New Request
              </Link>

              {/* User menu */}
              <div className="flex items-center gap-3 ml-4 pl-4 border-l border-slate-200">
                <Link
                  to="/profile"
                  className="flex items-center gap-2 text-slate-700 hover:text-slate-900 text-sm font-medium transition-all hover:bg-slate-100/80 px-3 py-2 rounded-lg"
                >
                  <div className="w-7 h-7 bg-gradient-to-br from-emerald-400 to-cyan-500 rounded-full flex items-center justify-center text-white text-xs font-semibold shadow-md">
                    {user?.name?.charAt(0).toUpperCase()}
                  </div>
                  {user?.name}
                </Link>
                <button
                  onClick={handleLogout}
                  className="text-slate-500 hover:text-slate-700 text-sm font-medium transition-colors"
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
