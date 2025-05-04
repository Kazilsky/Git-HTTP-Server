import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { FiGitBranch } from "react-icons/fi"
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
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex justify-between items-center">
          <h1 className="text-xl font-bold text-gray-900 flex items-center">
            <FiGitBranch className="mr-2 text-indigo-600" />
            GitClone
          </h1>
          <nav className="flex space-x-8">
            <Link to="/" className="text-indigo-600 font-medium">Repositories</Link>
            <a href="#" className="text-gray-500 hover:text-gray-700">Profile</a>
            <a href="#" className="text-gray-500 hover:text-gray-700">Settings</a>
          </nav>
        </div>
      </header>

 );
};

export default Header; 