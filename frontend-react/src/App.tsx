import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import './index.css';

// Import components
import Header from './components/Header';
import HomePage from './pages/HomePage';
import LoginPage from './pages/LoginPage';
import RepoPage from './pages/RepoPage';

interface User {
  id: number;
  username: string;
  email: string | null;
}

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
        <main className="flex-grow">
          { user ? <Header user={user} onLogout={handleLogout} /> : null } 
          <Routes>
          
            <Route path="/" element={
              user ? <HomePage /> : <Navigate to="/login" />
            } />
            <Route path="/login" element={
              user ? <Navigate to="/" /> : <LoginPage onLogin={handleLogin} />
            } />

            <Route path="/repo/:name" element={
              user ? <RepoPage/> : <LoginPage onLogin={handleLogin} />
            } />
            <Route path="/repo/:name/:branch" element={
              user ? <RepoPage/> : <LoginPage onLogin={handleLogin} />
            } />
 
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App; 