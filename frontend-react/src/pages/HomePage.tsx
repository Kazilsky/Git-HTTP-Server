import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { FiGitBranch, FiStar, FiClock, FiPlus } from 'react-icons/fi';
import { FaCodeBranch } from 'react-icons/fa';

interface Repository {
  id: number;
  name: string;
  description: string | null;
  owner_id: number;
  created_at: string;
  stars_count?: number;
  forks_count?: number;
}

interface ApiResponse {
  success: boolean;
  message: string | null;
  data: Repository[] | null;
}

const HomePage: React.FC = () => {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchRepositories = async () => {
      try {
        const response = await fetch('http://localhost:8000/api/repos', {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}`
          }
        });

        const data: ApiResponse = await response.json();
        
        if (data.success && data.data) {
          // Добавляем mock-данные для демонстрации
          const reposWithStats = data.data.map(repo => ({
            ...repo,
            stars_count: Math.floor(Math.random() * 100),
            forks_count: Math.floor(Math.random() * 50)
          }));
          setRepositories(reposWithStats);
        } else {
          setError(data.message || 'Failed to fetch repositories');
        }
      } catch (err) {
        setError('Error connecting to the server');
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    fetchRepositories();
  }, []);

  if (loading) {
    return (
      <div className="flex justify-center items-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center px-4">
        <div className="bg-red-50 border-l-4 border-red-500 p-4 max-w-md w-full">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">Error fetching repositories</h3>
              <div className="mt-2 text-sm text-red-700">
                <p>{error}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex justify-between items-center mb-8">
          <div>
            <h2 className="text-2xl font-bold text-gray-900">Your Repositories</h2>
            <p className="mt-1 text-sm text-gray-500">
              {repositories.length} {repositories.length === 1 ? 'repository' : 'repositories'}
            </p>
          </div>
          <Link 
            to="/create-repo" 
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <FiPlus className="mr-2" />
            New Repository
          </Link>
        </div>

        {repositories.length === 0 ? (
          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="px-4 py-5 sm:p-6 text-center">
              <svg
                className="mx-auto h-12 w-12 text-gray-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                aria-hidden="true"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                />
              </svg>
              <h3 className="mt-2 text-lg font-medium text-gray-900">No repositories</h3>
              <p className="mt-1 text-sm text-gray-500">
                Get started by creating a new repository.
              </p>
              <div className="mt-6">
                <Link
                  to="/create-repo"
                  className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  <FiPlus className="-ml-1 mr-2 h-5 w-5" />
                  New Repository
                </Link>
              </div>
            </div>
          </div>
        ) : (
          <div className="grid gap-6 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
            {repositories.map((repo) => (
              <div key={repo.id} className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-all duration-200">
                <Link to={`/repo/${repo.name}`} className="block">
                  <div className="px-4 py-5 sm:p-6">
                    <div className="flex items-center">
                      <div className="flex-shrink-0 bg-indigo-500 rounded-md p-2">
                        <FiGitBranch className="h-6 w-6 text-white" />
                      </div>
                      <div className="ml-4">
                        <h3 className="text-lg font-medium text-gray-900 truncate">{repo.name}</h3>
                        <p className="text-sm text-gray-500 mt-1">
                          {repo.description || 'No description provided'}
                        </p>
                      </div>
                    </div>
                    <div className="mt-6 flex justify-between items-center">
                      <div className="flex space-x-4">
                        <span className="inline-flex items-center text-sm text-gray-500">
                          <FiStar className="mr-1 text-yellow-500" />
                          {repo.stars_count}
                        </span>
                        <span className="inline-flex items-center text-sm text-gray-500">
                          <FaCodeBranch className="mr-1 text-gray-400" />
                          {repo.forks_count}
                        </span>
                      </div>
                      <span className="inline-flex items-center text-sm text-gray-500">
                        <FiClock className="mr-1 text-gray-400" />
                        {new Date(repo.created_at).toLocaleDateString('en-US', {
                          year: 'numeric',
                          month: 'short',
                          day: 'numeric'
                        })}
                      </span>
                    </div>
                  </div>
                </Link>
              </div>
            ))}
          </div>
        )}
      </main>

      <footer className="bg-white border-t border-gray-200 mt-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <p className="text-center text-sm text-gray-500">
            &copy; {new Date().getFullYear()} GitClone. All rights reserved.
          </p>
        </div>
      </footer>
    </div>
  );
};

export default HomePage;
