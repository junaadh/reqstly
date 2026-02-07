export function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen flex flex-col justify-center py-12 sm:px-6 lg:px-8 relative overflow-hidden">
      <div className="absolute inset-0 -z-10">
        <div className="absolute -top-24 -left-24 h-72 w-72 rounded-full bg-teal-300/30 blur-3xl"></div>
        <div className="absolute top-20 -right-20 h-80 w-80 rounded-full bg-sky-300/30 blur-3xl"></div>
        <div className="absolute bottom-10 left-1/3 h-64 w-64 rounded-full bg-emerald-200/25 blur-3xl"></div>
      </div>
      <div className="sm:mx-auto sm:w-full sm:max-w-md">
        <h1 className="text-center text-3xl font-display font-semibold text-slate-900">
          Reqstly
        </h1>
        <p className="mt-2 text-center text-sm text-slate-600">
          Request Management System
        </p>
      </div>

      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="surface-card-strong py-8 px-4 sm:px-10">
          { children }
        </div>
      </div>
    </div>
  );
}
