// components/RepositoryCard.tsx
import React from 'react';
import { Link } from 'react-router-dom';
import { FiStar, FiGitBranch, FiLock } from 'react-icons/fi';
import { FaCodeBranch } from 'react-icons/fa';

interface RepositoryCardProps {
  repository: {
    id: number;
    name: string;
    description: string | null;
    stars_count: number;
    forks_count: number;
    is_private: boolean;
    created_at: string;
  };
}

const RepositoryCard: React.FC<RepositoryCardProps> = ({ repository }) => {
  return (
    <div className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-all duration-200">
      <Link to={`/repo/${repository.name}`} className="block">
        <div className="px-4 py-5 sm:p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0 bg-indigo-500 rounded-md p-2">
              <FiGitBranch className="h-6 w-6 text-white" />
            </div>
            <div className="ml-4 flex-1 min-w-0">
              <div className="flex items-center">
                <h3 className="text-lg font-medium text-gray-900 truncate">{repository.name}</h3>
                {repository.is_private && (
                  <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                    <FiLock className="mr-1" /> Private
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-500 mt-1 truncate">
                {repository.description || 'No description provided'}
              </p>
            </div>
          </div>
          <div className="mt-6 flex justify-between items-center">
            <div className="flex space-x-4">
              <span className="inline-flex items-center text-sm text-gray-500">
                <FiStar className="mr-1 text-yellow-500" />
                {repository.stars_count}
              </span>
              <span className="inline-flex items-center text-sm text-gray-500">
                <FaCodeBranch className="mr-1 text-gray-400" />
                {repository.forks_count}
              </span>
            </div>
            <span className="inline-flex items-center text-xs text-gray-500">
              Created: {new Date(repository.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
      </Link>
    </div>
  );
};

export default RepositoryCard;