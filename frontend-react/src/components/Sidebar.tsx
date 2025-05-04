// components/Sidebar.tsx
import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { FiHome, FiGitBranch, FiFolder, FiUsers, FiSettings, FiBookmark } from 'react-icons/fi';

const Sidebar: React.FC = () => {
  const location = useLocation();

  const navItems = [
    { name: 'Dashboard', icon: FiHome, path: '/' },
    { name: 'Repositories', icon: FiGitBranch, path: '/repos' },
    { name: 'Projects', icon: FiFolder, path: '/projects' },
    { name: 'Teams', icon: FiUsers, path: '/teams' },
    { name: 'Bookmarks', icon: FiBookmark, path: '/bookmarks' },
    { name: 'Settings', icon: FiSettings, path: '/settings' },
  ];

  return (
    <div className="hidden md:flex md:flex-shrink-0">
      <div className="flex flex-col w-64 border-r border-gray-200 bg-white">
        <div className="h-0 flex-1 flex flex-col pt-5 pb-4 overflow-y-auto">
          <nav className="flex-1 px-2 space-y-1">
            {navItems.map((item) => (
              <Link
                key={item.name}
                to={item.path}
                className={`group flex items-center px-2 py-2 text-sm font-medium rounded-md ${
                  location.pathname === item.path
                    ? 'bg-indigo-50 text-indigo-700'
                    : 'text-gray-600 hover:text-indigo-600 hover:bg-gray-50'
                }`}
              >
                <item.icon
                  className={`mr-3 flex-shrink-0 h-5 w-5 ${
                    location.pathname === item.path
                      ? 'text-indigo-500'
                      : 'text-gray-400 group-hover:text-indigo-500'
                  }`}
                />
                {item.name}
              </Link>
            ))}
          </nav>
        </div>
        <div className="flex-shrink-0 flex border-t border-gray-200 p-4">
          <div className="flex items-center">
            <div className="h-9 w-9 rounded-full bg-indigo-100 flex items-center justify-center">
              <FiUser className="text-indigo-600" />
            </div>
            <div className="ml-3">
              <p className="text-sm font-medium text-gray-700">Username</p>
              <Link to="/settings" className="text-xs font-medium text-indigo-600 hover:text-indigo-500">View profile</Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;