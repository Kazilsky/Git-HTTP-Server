import React, { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';

interface RepoPageProps {
  onRepo: string;
}

interface Repository {
  id: number;
  name: string;
  description: string | null;
  owner_id: number;
  created_at: string;
}

interface Commit {
  id: string;
  message: string;
  author: string;
  date: string;
  hash: string;
}

interface File {
  name: string;
  type: 'file' | 'directory';
  size?: number;
  last_commit?: string;
}

interface ApiResponse<T> {
  success: boolean;
  message: string | null;
  data: T | null;
}

const RepoPage: React.FC<RepoPageProps> = ({ onRepo }) => {
  const { branch = 'main' } = useParams<{ branch?: string }>();
  const params = useParams();

  const [repository, setRepository] = useState<Repository | null>(null);
  const [files, setFiles] = useState<File[]>([]);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [branches, setBranches] = useState<string[]>([]);
  const [activeBranch, setActiveBranch] = useState<string>(branch);
  const [currentPath, setCurrentPath] = useState<string>('');
  const [activeTab, setActiveTab] = useState<'code' | 'commits' | 'branches'>('code');
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Загрузка данных о репозитории
  useEffect(() => {
    const fetchRepoDetails = async () => {
      try {
        setLoading(true);
        // Запрос данных о репозитории
        const repoResponse = await fetch(`http://localhost:8000/api/repos/${params["*"]}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}`
          }
        });
        
        const repoData: ApiResponse<Repository> = await repoResponse.json();

        console.log(repoData)
        
        if (!repoData.success) {
          throw new Error(repoData.message || 'Failed to fetch repository details');
        }
        
        setRepository(repoData.data);
        
        // Загрузка файлов
        await fetchFiles(activeBranch, currentPath);
        
        // Загрузка коммитов
        await fetchCommits(activeBranch);
        
        // Загрузка веток
        await fetchBranches();
        
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Error connecting to the server';
        setError(errorMessage);
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    fetchRepoDetails();
  }, [onRepo]);

  // Загрузка файлов при изменении ветки или пути
  useEffect(() => {
    if (repository) {
      fetchFiles(activeBranch, currentPath);
    }
  }, [activeBranch, currentPath]);

  // Загрузка файлов
  const fetchFiles = async (branch: string, path: string) => {
    try {
      const encodedPath = encodeURIComponent(path);
      const response = await fetch(`http://localhost:8000/api/repos/${onRepo}/contents?branch=${branch}&path=${encodedPath}`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}`
        }
      });
      
      const data: ApiResponse<File[]> = await response.json();
      
      if (data.success && data.data) {
        setFiles(data.data);
      } else {
        console.error('Failed to fetch files:', data.message);
      }
    } catch (err) {
      console.error('Error fetching files:', err);
    }
  };

  // Загрузка коммитов
  const fetchCommits = async (branch: string) => {
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${onRepo}/commits?branch=${branch}`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}`
        }
      });
      
      const data: ApiResponse<Commit[]> = await response.json();
      
      if (data.success && data.data) {
        setCommits(data.data);
      } else {
        console.error('Failed to fetch commits:', data.message);
      }
    } catch (err) {
      console.error('Error fetching commits:', err);
    }
  };

  // Загрузка веток
  const fetchBranches = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${onRepo}/branches`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}`
        }
      });
      
      const data: ApiResponse<string[]> = await response.json();
      
      if (data.success && data.data) {
        setBranches(data.data);
      } else {
        console.error('Failed to fetch branches:', data.message);
      }
    } catch (err) {
      console.error('Error fetching branches:', err);
    }
  };

  // Навигация по директориям
  const navigateToDirectory = (dirName: string) => {
    if (dirName === '..') {
      // Навигация на уровень выше
      const pathParts = currentPath.split('/').filter(Boolean);
      pathParts.pop();
      setCurrentPath(pathParts.length > 0 ? pathParts.join('/') : '');
    } else {
      // Навигация в подкаталог
      const newPath = currentPath ? `${currentPath}/${dirName}` : dirName;
      setCurrentPath(newPath);
    }
  };

  // Обработчик смены ветки
  const handleBranchChange = (branch: string) => {
    setActiveBranch(branch);
    setCurrentPath(''); // Сбрасываем текущий путь при смене ветки
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
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
    <div className="container mx-auto px-4 py-6">
      {/* Заголовок репозитория */}
      <div className="mb-8">
        <div className="flex items-center mb-4">
          <h1 className="text-3xl font-bold mr-4">{params["*"]}</h1>
          <div className="bg-gray-200 text-gray-700 px-3 py-1 rounded-full text-sm">
            {repository?.description || 'No description'}
          </div>
        </div>
        
        <div className="text-sm text-gray-600">
          Created: {repository?.created_at ? new Date(repository.created_at).toLocaleDateString() : 'Unknown'}
        </div>
      </div>

      {/* Селектор ветки */}
      <div className="mb-6 flex items-center">
        <div className="relative inline-block w-48 mr-4">
          <select
            className="block appearance-none w-full bg-white border border-gray-300 hover:border-gray-400 px-4 py-2 pr-8 rounded shadow"
            value={activeBranch}
            onChange={(e) => handleBranchChange(e.target.value)}
          >
            {branches.map(branch => (
              <option key={branch} value={branch}>{branch}</option>
            ))}
          </select>
          <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
            <svg className="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
              <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"/>
            </svg>
          </div>
        </div>
        
        {/* Кнопки действий */}
        <button className="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded-md text-sm mx-1">
          Clone
        </button>
        <button className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-md text-sm mx-1">
          Download ZIP
        </button>
      </div>

      {/* Навигация по вкладкам */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex">
          <button 
            onClick={() => setActiveTab('code')}
            className={`py-4 px-6 font-medium text-sm ${activeTab === 'code' 
              ? 'border-b-2 border-blue-500 text-blue-600' 
              : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'}`}
          >
            Code
          </button>
          <button 
            onClick={() => setActiveTab('commits')}
            className={`py-4 px-6 font-medium text-sm ${activeTab === 'commits' 
              ? 'border-b-2 border-blue-500 text-blue-600' 
              : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'}`}
          >
            Commits
          </button>
          <button 
            onClick={() => setActiveTab('branches')}
            className={`py-4 px-6 font-medium text-sm ${activeTab === 'branches' 
              ? 'border-b-2 border-blue-500 text-blue-600' 
              : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'}`}
          >
            Branches
          </button>
        </nav>
      </div>

      {/* Навигационная цепочка для текущего пути */}
      {activeTab === 'code' && (
        <div className="mb-4 text-sm">
          <span className="font-medium">Path:</span>
          <button 
            onClick={() => setCurrentPath('')}
            className="mx-1 text-blue-600 hover:underline"
          >
            {onRepo}
          </button>
          {currentPath && currentPath.split('/').filter(Boolean).map((part, index, parts) => (
            <React.Fragment key={index}>
              <span className="mx-1">/</span>
              <button 
                onClick={() => setCurrentPath(parts.slice(0, index + 1).join('/'))}
                className="text-blue-600 hover:underline"
              >
                {part}
              </button>
            </React.Fragment>
          ))}
        </div>
      )}

      {/* Содержимое активной вкладки */}
      <div className="bg-white rounded-lg shadow">
        {activeTab === 'code' && (
          <>
            {/* Отображение файлов */}
            {files.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                <p>This repository is empty.</p>
              </div>
            ) : (
              <table className="min-w-full">
                <thead className="bg-gray-50">
                  <tr>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Name
                    </th>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Last Commit
                    </th>
                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Size
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {currentPath && (
                    <tr className="hover:bg-gray-50 cursor-pointer" onClick={() => navigateToDirectory('..')}>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center">
                          <svg className="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                          </svg>
                          <span className="text-gray-600">..</span>
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap"></td>
                      <td className="px-6 py-4 whitespace-nowrap"></td>
                    </tr>
                  )}
                  
                  {files.map((file, index) => (
                    <tr 
                      key={index}
                      className="hover:bg-gray-50 cursor-pointer"
                      onClick={() => file.type === 'directory' ? navigateToDirectory(file.name) : null}
                    >
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center">
                          {file.type === 'directory' ? (
                            <svg className="h-5 w-5 text-blue-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                            </svg>
                          ) : (
                            <svg className="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>
                          )}
                          <span className={file.type === 'directory' ? 'text-blue-600' : ''}>
                            {file.name}
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {file.last_commit || '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {file.type === 'file' && file.size !== undefined 
                          ? `${(file.size < 1024 
                              ? file.size 
                              : file.size < 1024 * 1024 
                                ? (file.size / 1024).toFixed(1) + ' KB' 
                                : (file.size / (1024 * 1024)).toFixed(1) + ' MB')}`
                          : '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </>
        )}

        {activeTab === 'commits' && (
          <>
            {commits.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                <p>No commits found in this repository.</p>
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {commits.map((commit) => (
                  <li key={commit.id} className="px-6 py-4 hover:bg-gray-50">
                    <div className="flex justify-between items-start">
                      <div>
                        <h3 className="text-lg font-medium text-gray-900">{commit.message}</h3>
                        <p className="text-sm text-gray-600">
                          <span className="font-medium">{commit.author}</span> committed on {new Date(commit.date).toLocaleString()}
                        </p>
                      </div>
                      <div className="flex items-center">
                        <span className="bg-gray-200 px-3 py-1 rounded-md text-sm font-mono">{commit.hash.substring(0, 7)}</span>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </>
        )}

        {activeTab === 'branches' && (
          <>
            {branches.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                <p>No branches found in this repository.</p>
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {branches.map((branch) => (
                  <li key={branch} className="px-6 py-4 hover:bg-gray-50">
                    <div className="flex justify-between items-center">
                      <div className="flex items-center">
                        <svg className="h-5 w-5 text-gray-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
                        </svg>
                        <span className="text-lg font-medium">{branch}</span>
                        {branch === activeBranch && (
                          <span className="ml-2 bg-green-100 text-green-800 text-xs px-2 py-1 rounded">Current</span>
                        )}
                      </div>
                      <button 
                        onClick={() => handleBranchChange(branch)}
                        className="text-blue-600 hover:text-blue-800"
                      >
                        Switch
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default RepoPage;
