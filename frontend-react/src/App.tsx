import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import './index.css';

// Import components
import Header from './components/Header';
import HomePage from './pages/HomePage';
import LoginPage from './pages/LoginPage';
import RepoPage from './pages/RepoPage';
import PullRequestPage from './pages/PullRequestPage';

/**
 * Interface representing a user in the system
 * @interface User
 * @property {number} id - The user's unique identifier
 * @property {string} username - The user's username
 * @property {string|null} email - The user's email address (optional)
 */
interface User {
  id: number;
  username: string;
  email: string | null;
}

/**
 * Main application component that handles routing and authentication state
 * 
 * @component
 * @description
 * The App component is the root component of the application that:
 * - Manages user authentication state
 * - Handles routing between different pages
 * - Provides the main layout structure (header, main content, footer)
 * - Persists user session in localStorage
 * 
 * @example
 * ```tsx
 * <App />
 * ```
 */

function App() {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    // Check if user is already logged in
    const storedUser = localStorage.getItem('user');
    if (storedUser) {
      try {
        setUser(JSON.parse(storedUser));
      } catch (e) {
        console.error('Failed to parse stored user', e);
        localStorage.removeItem('user');
      }
    }
  }, []);

  const handleLogin = (userData: User) => {
    setUser(userData);
  };

  const handleLogout = () => {
    localStorage.removeItem('user');
    setUser(null);
  };

  return (
    <Router>
      <div className="min-h-screen flex flex-col bg-gray-100">
        <Header user={user} onLogout={handleLogout} />
        <main className="flex-grow">
          <Routes>
            <Route path="/" element={
              user ? <HomePage /> : <Navigate to="/login" />
            } />
            <Route path="/login" element={
              user ? <Navigate to="/" /> : <LoginPage onLogin={handleLogin} />
            } />
            <Route path="/repo/:repoName" element={<RepoPage />} />
            <Route path="/repo/:repoName/pull/:pullId" element={<PullRequestPage />} />
          </Routes>
        </main>
        <footer className="bg-gray-800 text-white py-4">
          <div className="container mx-auto px-4 text-center">
            <p>Git HTTP Server &copy; {new Date().getFullYear()}</p>
          </div>
        </footer>
      </div>
    </Router>
  );
}

export default App;
