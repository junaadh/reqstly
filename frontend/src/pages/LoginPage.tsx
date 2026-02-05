import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

const API_BASE =
  import.meta.env.VITE_API_BASE_URL ??
  import.meta.env.VITE_API_URL ??
  'http://localhost:3000'

async function postJson<T>(path: string, payload: T) {
  const response = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    const message = await response.text()
    throw new Error(message || 'Request failed')
  }

  return response.json().catch(() => ({}))
}

export function LoginPage() {
  const navigate = useNavigate()
  const [loginLoading, setLoginLoading] = useState(false)
  const [signupLoading, setSignupLoading] = useState(false)
  const [loginForm, setLoginForm] = useState({ email: '', password: '' })
  const [signupForm, setSignupForm] = useState({ name: '', email: '', password: '' })

  const handleLogin = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    if (!loginForm.email || !loginForm.password) {
      toast.error('Please enter your email and password.')
      return
    }

    try {
      setLoginLoading(true)
      await postJson('/auth/password/login', loginForm)
      toast.success('Welcome back!')
      navigate('/')
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Login failed.'
      toast.error(message)
    } finally {
      setLoginLoading(false)
    }
  }

  const handleSignup = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    if (!signupForm.name || !signupForm.email || !signupForm.password) {
      toast.error('Please complete all fields.')
      return
    }

    try {
      setSignupLoading(true)
      await postJson('/auth/password/signup', signupForm)
      toast.success('Account created successfully.')
      navigate('/')
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Signup failed.'
      toast.error(message)
    } finally {
      setSignupLoading(false)
    }
  }

  const handleWorkInProgress = (label: string) => {
    toast.message(`${label} is a work in progress.`, {
      description: 'We are wiring this up next.',
    })
  }

  return (
    <div className="min-h-screen bg-hero-gradient">
      <div className="container flex min-h-screen items-center py-12">
        <div className="grid w-full gap-10 lg:grid-cols-[1.1fr_0.9fr]">
          <section className="space-y-6">
            <p className="text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground">
              Cross-department intake
            </p>
            <h1 className="text-4xl font-semibold leading-tight md:text-5xl">
              Unblock requests from Finance, Legal, People Ops, and Security in one flow.
            </h1>
            <p className="max-w-xl text-base text-muted-foreground">
              Reqstly brings every handoff into a single workspace with clear ownership, SLA visibility, and audit-ready
              records.
            </p>
            <div className="grid gap-4 md:grid-cols-2">
              {[
                {
                  title: 'Unified intake',
                  body: 'One request form routed to the right team instantly.',
                },
                {
                  title: 'Clear accountability',
                  body: 'Track status, owners, and escalations without follow-ups.',
                },
                {
                  title: 'Approval trails',
                  body: 'Capture decisions and compliance context in real time.',
                },
                {
                  title: 'Live insights',
                  body: 'Understand bottlenecks across every department queue.',
                },
              ].map((item) => (
                <div key={item.title} className="rounded-3xl border bg-card/80 p-5 shadow-sm">
                  <p className="text-sm font-semibold">{item.title}</p>
                  <p className="mt-2 text-sm text-muted-foreground">{item.body}</p>
                </div>
              ))}
            </div>
          </section>

          <Card className="shadow-glow">
            <CardHeader>
              <CardTitle>Sign in</CardTitle>
              <CardDescription>Use password auth today. Passkeys and Azure AD are next.</CardDescription>
            </CardHeader>
            <CardContent>
              <Tabs defaultValue="login">
                <TabsList className="w-full">
                  <TabsTrigger value="login" className="flex-1">
                    Sign in
                  </TabsTrigger>
                  <TabsTrigger value="signup" className="flex-1">
                    Sign up
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="login">
                  <form className="space-y-4" onSubmit={handleLogin}>
                    <div className="space-y-2">
                      <Label htmlFor="login-email">Work email</Label>
                      <Input
                        id="login-email"
                        type="email"
                        placeholder="you@company.com"
                        value={loginForm.email}
                        onChange={(event) =>
                          setLoginForm((prev) => ({ ...prev, email: event.target.value }))
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="login-password">Password</Label>
                      <Input
                        id="login-password"
                        type="password"
                        placeholder="Enter your password"
                        value={loginForm.password}
                        onChange={(event) =>
                          setLoginForm((prev) => ({ ...prev, password: event.target.value }))
                        }
                      />
                    </div>
                    <Button type="submit" className="w-full" disabled={loginLoading}>
                      {loginLoading ? 'Signing in...' : 'Sign in'}
                    </Button>
                  </form>
                </TabsContent>

                <TabsContent value="signup">
                  <form className="space-y-4" onSubmit={handleSignup}>
                    <div className="space-y-2">
                      <Label htmlFor="signup-name">Full name</Label>
                      <Input
                        id="signup-name"
                        type="text"
                        placeholder="Alex Morgan"
                        value={signupForm.name}
                        onChange={(event) =>
                          setSignupForm((prev) => ({ ...prev, name: event.target.value }))
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="signup-email">Work email</Label>
                      <Input
                        id="signup-email"
                        type="email"
                        placeholder="you@company.com"
                        value={signupForm.email}
                        onChange={(event) =>
                          setSignupForm((prev) => ({ ...prev, email: event.target.value }))
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="signup-password">Password</Label>
                      <Input
                        id="signup-password"
                        type="password"
                        placeholder="Create a password"
                        value={signupForm.password}
                        onChange={(event) =>
                          setSignupForm((prev) => ({ ...prev, password: event.target.value }))
                        }
                      />
                    </div>
                    <Button type="submit" className="w-full" disabled={signupLoading}>
                      {signupLoading ? 'Creating account...' : 'Create account'}
                    </Button>
                  </form>
                </TabsContent>
              </Tabs>

              <Separator className="my-6" />

              <div className="space-y-3">
                <Button
                  variant="outline"
                  type="button"
                  className="w-full"
                  onClick={() => handleWorkInProgress('Passkey sign-in')}
                >
                  Continue with passkey
                </Button>
                <Button
                  variant="outline"
                  type="button"
                  className="w-full"
                  onClick={() => handleWorkInProgress('Azure AD sign-in')}
                >
                  Continue with Azure AD
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
