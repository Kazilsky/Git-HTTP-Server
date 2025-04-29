import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

interface Repository {
  id: number;
  name: string;
  description: string | null;
  owner_id: number;
  created_at: string;
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
            'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
          }
        });

        const data: ApiResponse = await response.json();
        
        if (data.success && data.data) {
          setRepositories(data.data);
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
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-gray-900"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
        <strong className="font-bold">Error:</strong>
        <span className="block sm:inline"> {error}</span>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Your Repositories</h1>
        <Link to="/create-repo" className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
          New Repository
        </Link>
      </div>

      {repositories.length === 0 ? (
        <div className="bg-gray-100 rounded-md p-6 text-center">
          <p className="text-gray-600">No repositories found. Create your first repository to get started.</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {repositories.map((repo) => (
            <div key={repo.id} className="border rounded-md p-4 hover:shadow-md transition-shadow">
              <Link to={`/repo/${repo.name}`} className="text-blue-600 hover:text-blue-800 font-semibold text-lg">
                {repo.name}
              </Link>
              <p className="text-gray-600 mt-2">{repo.description || 'No description'}</p>
              <p className="text-gray-400 text-sm mt-2">Created: {new Date(repo.created_at).toLocaleDateString()}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default HomePage; 