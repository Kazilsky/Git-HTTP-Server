import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';

interface User {
  id: number;
  username: string;
  email: string | null;
}

interface HeaderProps {
  user: User | null;
  onLogout: () => void;
}

const Header: React.FC<HeaderProps> = ({ user, onLogout }) => {
  const [menuOpen, setMenuOpen] = useState(false);
  const navigate = useNavigate();

  const toggleMenu = () => {
    setMenuOpen(!menuOpen);
  };

  const handleLogout = () => {
    onLogout();
    navigate('/login');
  };

  return (
    <header className="bg-gray-800 text-white">
      <div className="container mx-auto px-4 py-3">
        <div className="flex justify-between items-center">
          <div className="flex items-center">
            <Link to="/" className="text-xl font-bold">Git HTTP Server</Link>
            <nav className="ml-8 hidden md:flex space-x-4">
              <Link to="/" className="hover:text-gray-300">Repositories</Link>
              <Link to="/docs" className="hover:text-gray-300">Documentation</Link>
            </nav>
          </div>
          
          <div className="relative">
            {user ? (
              <>
                <button 
                  onClick={toggleMenu}
                  className="flex items-center focus:outline-none"
                >
                  <span className="mr-2">{user.username}</span>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </button>
                
                {menuOpen && (
                  <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 text-gray-700">
                    <Link 
                      to={`/user/${user.username}`} 
                      className="block px-4 py-2 hover:bg-gray-100"
                      onClick={() => setMenuOpen(false)}
                    >
                      Profile
                    </Link>
                    <Link 
                      to="/settings" 
                      className="block px-4 py-2 hover:bg-gray-100"
                      onClick={() => setMenuOpen(false)}
                    >
                      Settings
                    </Link>
                    <button 
                      className="block w-full text-left px-4 py-2 hover:bg-gray-100"
                      onClick={handleLogout}
                    >
                      Logout
                    </button>
                  </div>
                )}
              </>
            ) : (
              <div className="space-x-4">
                <Link to="/login" className="hover:text-gray-300">Login</Link>
                <Link to="/register" className="bg-blue-600 hover:bg-blue-700 px-3 py-1 rounded">Register</Link>
              </div>
            )}
          </div>
        </div>
        
        {/* Mobile menu */}
        <div className="md:hidden mt-2">
          <button onClick={toggleMenu} className="text-white focus:outline-none">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          
          {menuOpen && (
            <nav className="mt-2 space-y-2">
              <Link to="/" className="block hover:text-gray-300">Repositories</Link>
              <Link to="/docs" className="block hover:text-gray-300">Documentation</Link>
            </nav>
          )}
        </div>
      </div>
    </header>
  );
};

export default Header; 