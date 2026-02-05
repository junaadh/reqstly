import { useAuth } from '@/contexts/AuthContext'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { User, Key } from 'lucide-react'

export function ProfilePage() {
  const { user, logout } = useAuth()

  return (
    <div className="p-8 animate-fade-in">
      {/* Header */}
      <div className="mb-8">
        <h1 className="font-display text-3xl font-bold tracking-tight">
          Profile
        </h1>
        <p className="mt-2 text-muted-foreground">
          Manage your account settings and preferences
        </p>
      </div>

      <div className="max-w-2xl space-y-6">
        {/* User Information */}
        <Card className="animate-slide-up">
          <CardHeader>
            <CardTitle>User Information</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center gap-4">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary">
                <span className="text-2xl font-bold text-primary-foreground">
                  {user?.name.charAt(0).toUpperCase()}
                </span>
              </div>
              <div>
                <h3 className="text-lg font-semibold">{user?.name}</h3>
                <p className="text-sm text-muted-foreground">{user?.email}</p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Authentication Methods */}
        <Card className="animate-slide-up">
          <CardHeader>
            <CardTitle>Authentication Methods</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between p-4 border rounded-lg">
              <div className="flex items-center gap-3">
                <User className="h-5 w-5 text-muted-foreground" />
                <div>
                  <p className="font-medium">Password</p>
                  <p className="text-sm text-muted-foreground">Enabled</p>
                </div>
              </div>
              <Button variant="outline" size="sm">
                Change
              </Button>
            </div>

            <div className="flex items-center justify-between p-4 border rounded-lg">
              <div className="flex items-center gap-3">
                <Key className="h-5 w-5 text-muted-foreground" />
                <div>
                  <p className="font-medium">Passkey</p>
                  <p className="text-sm text-muted-foreground">Not configured</p>
                </div>
              </div>
              <Button variant="outline" size="sm">
                Add
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Danger Zone */}
        <Card className="animate-slide-up border-destructive">
          <CardHeader>
            <CardTitle className="text-destructive">Danger Zone</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-4">
              Once you log out, you'll need to sign in again to access your account.
            </p>
            <Button variant="destructive" onClick={logout}>
              Logout
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
