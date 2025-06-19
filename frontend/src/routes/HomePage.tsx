import { Component, createSignal } from 'solid-js';
import { A } from '@solidjs/router';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

export const HomePage: Component = () => {
  const [email, setEmail] = createSignal('');
  const [name, setName] = createSignal('');
  const [message, setMessage] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [submitted, setSubmitted] = createSignal(false);

  const handleWaitlistSubmit = async (e: Event) => {
    e.preventDefault();
    if (!email().trim()) return;

    setLoading(true);
    try {
      // TODO: Replace with actual API call when backend is ready
      // await apiClient.joinWaitlist({ email: email(), name: name(), message: message() });
      console.log('Waitlist signup:', { email: email(), name: name(), message: message() });
      setSubmitted(true);
    } catch (error) {
      console.error('Waitlist signup failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="min-h-screen bg-gradient-to-br from-green-50 to-blue-50">
      {/* Navigation */}
      <nav class="flex items-center justify-between p-6 max-w-7xl mx-auto">
        <div class="flex items-center space-x-2">
          <div class="w-8 h-8 bg-green-600 rounded-lg flex items-center justify-center">
            <span class="text-white font-bold text-lg">🌱</span>
          </div>
          <span class="text-xl font-bold text-gray-900">Planty</span>
        </div>
        <div class="flex items-center space-x-4">
          <A href="/login" class="text-gray-600 hover:text-gray-900 font-medium">
            Sign In
          </A>
          <Button variant="outline" size="sm">
            <A href="/invite" class="no-underline">
              Get Started
            </A>
          </Button>
        </div>
      </nav>

      {/* Hero Section */}
      <section class="max-w-7xl mx-auto px-6 py-20">
        <div class="grid lg:grid-cols-2 gap-12 items-center">
          <div>
            <h1 class="text-5xl font-bold text-gray-900 leading-tight mb-6">
              Never forget to water your plants again
            </h1>
            <p class="text-xl text-gray-600 mb-8 leading-relaxed">
              Planty helps you track your plant care schedule, monitor growth metrics, 
              and build healthy habits that keep your green friends thriving.
            </p>
            <div class="flex flex-col sm:flex-row gap-4">
              <Button size="lg" class="text-lg px-8">
                <A href="/invite" class="no-underline">
                  Get Started
                </A>
              </Button>
              <Button variant="outline" size="lg" class="text-lg px-8">
                Learn More
              </Button>
            </div>
          </div>
          <div class="lg:text-right">
            {/* Placeholder for app screenshot */}
            <div class="bg-white rounded-xl shadow-2xl p-8 border">
              <div class="bg-gray-100 rounded-lg h-96 flex items-center justify-center">
                <div class="text-center">
                  <div class="text-6xl mb-4">📱</div>
                  <p class="text-gray-500 font-medium">App Screenshot</p>
                  <p class="text-gray-400 text-sm">Coming Soon</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section class="bg-white py-20">
        <div class="max-w-7xl mx-auto px-6">
          <div class="text-center mb-16">
            <h2 class="text-3xl font-bold text-gray-900 mb-4">
              Everything you need to care for your plants
            </h2>
            <p class="text-xl text-gray-600 max-w-3xl mx-auto">
              From beginner-friendly reminders to advanced growth tracking, 
              Planty adapts to your gardening experience level.
            </p>
          </div>
          
          <div class="grid md:grid-cols-3 gap-8">
            <div class="text-center p-6">
              <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <span class="text-2xl">💧</span>
              </div>
              <h3 class="text-xl font-semibold text-gray-900 mb-3">Smart Watering Reminders</h3>
              <p class="text-gray-600">
                Customizable schedules based on your plant types, pot sizes, and environmental conditions.
              </p>
            </div>
            
            <div class="text-center p-6">
              <div class="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <span class="text-2xl">📊</span>
              </div>
              <h3 class="text-xl font-semibold text-gray-900 mb-3">Growth Tracking</h3>
              <p class="text-gray-600">
                Monitor height, leaf count, health metrics, and watch your plants flourish over time.
              </p>
            </div>
            
            <div class="text-center p-6">
              <div class="w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <span class="text-2xl">📅</span>
              </div>
              <h3 class="text-xl font-semibold text-gray-900 mb-3">Care Calendar</h3>
              <p class="text-gray-600">
                Integrated calendar view with feeding schedules, repotting reminders, and seasonal care tips.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Screenshots Gallery */}
      <section class="py-20 bg-gray-50">
        <div class="max-w-7xl mx-auto px-6">
          <div class="text-center mb-16">
            <h2 class="text-3xl font-bold text-gray-900 mb-4">
              See Planty in action
            </h2>
            <p class="text-xl text-gray-600">
              Simple, intuitive design that makes plant care enjoyable
            </p>
          </div>
          
          <div class="grid md:grid-cols-3 gap-8">
            {/* Placeholder Screenshots */}
            <div class="bg-white rounded-xl shadow-lg overflow-hidden">
              <div class="bg-gradient-to-br from-green-100 to-green-200 h-64 flex items-center justify-center">
                <div class="text-center">
                  <div class="text-4xl mb-2">🌿</div>
                  <p class="text-gray-600 font-medium">Plant List View</p>
                </div>
              </div>
              <div class="p-6">
                <h3 class="font-semibold text-gray-900 mb-2">Your Plant Collection</h3>
                <p class="text-gray-600 text-sm">
                  Beautiful grid view of all your plants with quick status indicators.
                </p>
              </div>
            </div>
            
            <div class="bg-white rounded-xl shadow-lg overflow-hidden">
              <div class="bg-gradient-to-br from-blue-100 to-blue-200 h-64 flex items-center justify-center">
                <div class="text-center">
                  <div class="text-4xl mb-2">📈</div>
                  <p class="text-gray-600 font-medium">Growth Charts</p>
                </div>
              </div>
              <div class="p-6">
                <h3 class="font-semibold text-gray-900 mb-2">Track Progress</h3>
                <p class="text-gray-600 text-sm">
                  Visual charts showing your plant's growth and health metrics over time.
                </p>
              </div>
            </div>
            
            <div class="bg-white rounded-xl shadow-lg overflow-hidden">
              <div class="bg-gradient-to-br from-purple-100 to-purple-200 h-64 flex items-center justify-center">
                <div class="text-center">
                  <div class="text-4xl mb-2">🔔</div>
                  <p class="text-gray-600 font-medium">Care Reminders</p>
                </div>
              </div>
              <div class="p-6">
                <h3 class="font-semibold text-gray-900 mb-2">Never Miss Care</h3>
                <p class="text-gray-600 text-sm">
                  Smart notifications that help you stay on top of watering and feeding schedules.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Waitlist Section */}
      <section class="py-20 bg-green-600">
        <div class="max-w-4xl mx-auto px-6 text-center">
          <h2 class="text-3xl font-bold text-white mb-4">
            Join the Planty Waitlist
          </h2>
          <p class="text-xl text-green-100 mb-8">
            Be among the first to experience the future of plant care. 
            We'll notify you as soon as Planty is ready!
          </p>
          
          {submitted() ? (
            <div class="bg-white rounded-lg p-8 max-w-md mx-auto">
              <div class="text-4xl mb-4">✅</div>
              <h3 class="text-xl font-semibold text-gray-900 mb-2">You're on the list!</h3>
              <p class="text-gray-600">
                Thank you for your interest. We'll be in touch soon with updates and early access.
              </p>
            </div>
          ) : (
            <form onSubmit={handleWaitlistSubmit} class="max-w-md mx-auto">
              <div class="space-y-4">
                <Input
                  type="email"
                  placeholder="Your email address"
                  value={email()}
                  onInput={(e) => setEmail(e.currentTarget.value)}
                  required
                  class="bg-white"
                />
                <Input
                  type="text"
                  placeholder="Your name (optional)"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  class="bg-white"
                />
                <textarea
                  placeholder="Tell us about your plant care experience (optional)"
                  value={message()}
                  onInput={(e) => setMessage(e.currentTarget.value)}
                  rows="3"
                  class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500"
                />
                <Button
                  type="submit"
                  size="lg"
                  class="w-full bg-white text-green-600 hover:bg-gray-50"
                  loading={loading()}
                  disabled={!email().trim()}
                >
                  Join the Waitlist
                </Button>
              </div>
            </form>
          )}
        </div>
      </section>

      {/* Footer */}
      <footer class="bg-gray-900 py-12">
        <div class="max-w-7xl mx-auto px-6">
          <div class="flex flex-col md:flex-row justify-between items-center">
            <div class="flex items-center space-x-2 mb-4 md:mb-0">
              <div class="w-8 h-8 bg-green-600 rounded-lg flex items-center justify-center">
                <span class="text-white font-bold text-lg">🌱</span>
              </div>
              <span class="text-xl font-bold text-white">Planty</span>
            </div>
            <div class="flex items-center space-x-6">
              <a href="#" class="text-gray-400 hover:text-white">Privacy</a>
              <a href="#" class="text-gray-400 hover:text-white">Terms</a>
              <a href="#" class="text-gray-400 hover:text-white">Contact</a>
            </div>
          </div>
          <div class="mt-8 pt-8 border-t border-gray-800 text-center">
            <p class="text-gray-400">
              © 2024 Planty. All rights reserved. Built with 🌱 for plant lovers.
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
};