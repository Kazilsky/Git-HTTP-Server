import React, { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';

interface RepoPageProps {
  onRepo: string;
}

interface Repository {
  branches: string[];
  repo: {
    id: number | any;
    name: string | any;
    description: string | null;
    owner: {};
    created_at: string;
  };
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
  data: T;
}

const RepoPage: React.FC<RepoPageProps> = () => {
  const { branch = 'main' } = useParams<{ branch?: string }>();
  const params = useParams();
  const projectName = params["name"] || '';

  const [repository, setRepository] = useState<Repository | null>(null);
  const [files, setFiles] = useState<File[]>([]);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [branches, setBranches] = useState<string[]>([]);
  const [activeBranch, setActiveBranch] = useState<string>(branch);
  const [currentPath, setCurrentPath] = useState<string>('');
  const [activeTab, setActiveTab] = useState<'code' | 'commits' | 'branches' | 'issues' | 'wiki'>('code');
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [showCloneOptions, setShowCloneOptions] = useState<boolean>(false);
  const [readmeContent, setReadmeContent] = useState<string | null>(null);

  // Загрузка данных о репозитории
  useEffect(() => {
    const fetchRepoDetails = async () => {
      try {
        setLoading(true);
        // Запрос данных о репозитории
        const repoResponse = await fetch(`http://localhost:8000/api/repos/${projectName}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}`
          }
        });
        
        const repoData: ApiResponse<Repository> = await repoResponse.json();

        if (!repoData.success) {
          throw new Error(repoData.message || 'Failed to fetch repository details');
        }
        setBranches(repoData.data.branches);
        setRepository(repoData.data);
        
        // Загрузка файлов
        await fetchFiles(activeBranch, currentPath);
        
        // Загрузка коммитов
        await fetchCommits(activeBranch);
        
        // Загрузка веток
        //await fetchBranches();
        
        // Проверка наличия README
        checkForReadme(activeBranch);
        
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Error connecting to the server';
        setError(errorMessage);
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    fetchRepoDetails();
  }, [projectName]);

  // Загрузка файлов при изменении ветки или пути
  useEffect(() => {
    if (repository) {
      fetchFiles(activeBranch, currentPath);
      checkForReadme(activeBranch);
    }
  }, [activeBranch, currentPath]);

  // Загрузка README
  const checkForReadme = async (branch: string) => {
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${projectName}/readme?branch=${branch}`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}`
        }
      });
      
      if (response.ok) {
        const data = await response.json();
        if (data.success && data.data) {
          setReadmeContent(data.data);
        } else {
          setReadmeContent(null);
        }
      } else {
        setReadmeContent(null);
      }
    } catch (err) {
      console.error('Error fetching README:', err);
      setReadmeContent(null);
    }
  };

  // Загрузка файлов
  const fetchFiles = async (branch: string, path: string) => {
    try {
      const encodedPath = encodeURIComponent(path);
      const response = await fetch(
        `http://localhost:8000/api/repos/${projectName}/contents?branch=${branch}&path=${encodedPath}`, 
        {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}`
          }
        }
      );
      
      const data: ApiResponse<File[]> = await response.json();
      
      if (data.success && data.data) {
        setFiles(data.data.sort((a, b) => {
          // Directories first, then files
          if (a.type === 'directory' && b.type === 'file') return -1;
          if (a.type === 'file' && b.type === 'directory') return 1;
          return a.name.localeCompare(b.name);
        }));
      } else {
        console.error('Failed to fetch files:', data.message);
        setFiles([]);
      }
    } catch (err) {
      console.error('Error fetching files:', err);
      setFiles([]);
    }
  };

  // Загрузка коммитов
  const fetchCommits = async (branch: string) => {
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${projectName}/commits?branch=${branch}`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}`
        }
      });
      
      const data: ApiResponse<Commit[]> = await response.json();
      
      if (data.success && data.data) {
        setCommits(data.data);
      } else {
        console.error('Failed to fetch commits:', data.message);
        setCommits([]);
      }
    } catch (err) {
      console.error('Error fetching commits:', err);
      setCommits([]);
    }
  };

  // Загрузка веток
  /*const fetchBranches = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${projectName}/branches`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}`
        }
      });
      
      const data: ApiResponse<string[]> = await response.json();
      
      if (data.success && data.data) {
        setBranches(data.data);
      } else {
        console.error('Failed to fetch branches:', data.message);
        setBranches([]);
      }
    } catch (err) {
      console.error('Error fetching branches:', err);
      setBranches([]);
    }
  };*/

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

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - date.getTime());
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) {
      return 'today';
    } else if (diffDays === 1) {
      return 'yesterday';
    } else if (diffDays < 30) {
      return `${diffDays} days ago`;
    } else {
      return date.toLocaleDateString('en-US', {
        day: 'numeric',
        month: 'short',
        year: 'numeric'
      });
    }
  };

  const formatFileSize = (size: number | undefined) => {
    if (size === undefined) return '-';
    
    if (size < 1024) {
      return `${size} B`;
    } else if (size < 1024 * 1024) {
      return `${(size / 1024).toFixed(1)} KB`;
    } else {
      return `${(size / (1024 * 1024)).toFixed(1)} MB`;
    }
  };

  const getFileIcon = (fileName: string, type: 'file' | 'directory') => {
    if (type === 'directory') {
      return (
        <svg className="h-5 w-5 text-blue-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
      );
    }
    
    const extension = fileName.split('.').pop()?.toLowerCase();
    
    // Based on file extension, return appropriate icon
    if (['js', 'jsx', 'ts', 'tsx'].includes(extension || '')) {
      return <div className="w-5 h-5 mr-2 text-yellow-500 flex items-center justify-center text-xs font-bold">JS</div>;
    } else if (['html', 'htm'].includes(extension || '')) {
      return <div className="w-5 h-5 mr-2 text-orange-500 flex items-center justify-center text-xs font-bold">HTML</div>;
    } else if (['css', 'scss', 'sass'].includes(extension || '')) {
      return <div className="w-5 h-5 mr-2 text-blue-400 flex items-center justify-center text-xs font-bold">CSS</div>;
    } else if (['md', 'markdown'].includes(extension || '')) {
      return <div className="w-5 h-5 mr-2 text-gray-600 flex items-center justify-center text-xs font-bold">MD</div>;
    } else if (['json'].includes(extension || '')) {
      return <div className="w-5 h-5 mr-2 text-green-500 flex items-center justify-center text-xs font-bold">{};</div>;
    } else if (['py'].includes(extension || '')) {
      return <div className="w-5 h-5 mr-2 text-blue-600 flex items-center justify-center text-xs font-bold">PY</div>;
    } else if (['jpg', 'jpeg', 'png', 'gif', 'svg'].includes(extension || '')) {
      return (
        <svg className="h-5 w-5 text-pink-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
      );
    }
    
    // Default file icon
    return (
      <svg className="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
    );
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-indigo-500"></div>
        <span className="ml-3 text-lg text-gray-700">Loading project...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-8">
        <div className="bg-red-50 border-l-4 border-red-500 p-4 rounded shadow-sm mb-6">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd"/>
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm leading-5 font-medium text-red-800">
                Error loading project
              </h3>
              <div className="mt-1 text-sm leading-5 text-red-700">
                {error}
              </div>
              <div className="mt-4">
                <div className="-mx-2 -my-1.5 flex">
                  <button 
                    onClick={() => window.location.reload()}
                    className="px-2 py-1.5 rounded-md text-sm leading-5 font-medium text-red-800 hover:bg-red-100 focus:outline-none focus:bg-red-100"
                  >
                    Try again
                  </button>
                  <Link to="/" className="ml-3 px-2 py-1.5 rounded-md text-sm leading-5 font-medium text-red-800 hover:bg-red-100 focus:outline-none focus:bg-red-100">
                    Return home
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gray-50 min-h-screen pb-10">
      {/* Project header */}
      <div className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between py-4">
            <div>
              <div className="flex items-center">
                <svg className="h-6 w-6 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
                <h1 className="ml-2 text-xl font-bold text-gray-900">{projectName}</h1>
                
                {repository?.repo.description && (
                  <p className="ml-4 text-sm text-gray-600 hidden md:block">{repository.repo.description}</p>
                )}
              </div>
              
              {repository?.repo.description && (
                <p className="mt-1 text-sm text-gray-600 md:hidden">{repository.repo.description}</p>
              )}
            </div>
            
            <div className="flex items-center space-x-3">
              <div className="relative">
                <button 
                  onClick={() => setShowCloneOptions(!showCloneOptions)}
                  className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  <svg className="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                  </svg>
                  Clone
                </button>
                
                {showCloneOptions && (
                  <div className="origin-top-right absolute right-0 mt-2 w-64 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 z-10">
                    <div className="py-2 px-3 border-b border-gray-200">
                      <h3 className="text-sm font-medium text-gray-900">Clone repository</h3>
                    </div>
                    <div className="p-3">
                      <div className="flex items-center justify-between bg-gray-50 rounded-md p-2">
                        <code className="text-xs text-gray-800">https://github.com/username/{projectName}.git</code>
                        <button className="text-gray-500 hover:text-gray-700">
                          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                          </svg>
                        </button>
                      </div>
                      <div className="mt-2 flex justify-between">
                        <button className="text-sm text-indigo-600 hover:text-indigo-800 font-medium">
                          Open in Desktop
                        </button>
                        <button className="text-sm text-indigo-600 hover:text-indigo-800 font-medium">
                          Download ZIP
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </div>
              
              <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                <svg className="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
                Star
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Project nav tabs */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex -mb-px overflow-x-auto">
            <button 
              onClick={() => setActiveTab('code')}
              className={`whitespace-nowrap py-4 px-3 border-b-2 font-medium text-sm ${
                activeTab === 'code' 
                  ? 'border-indigo-500 text-indigo-600' 
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <svg className="h-5 w-5 inline mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
              </svg>
              Code
            </button>
            
            <button 
              onClick={() => setActiveTab('issues')}
              className={`whitespace-nowrap py-4 px-3 border-b-2 font-medium text-sm ${
                activeTab === 'issues' 
                  ? 'border-indigo-500 text-indigo-600' 
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <svg className="h-5 w-5 inline mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              Issues
              <span className="ml-1 px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-900">0</span>
            </button>
            
            <button 
              onClick={() => setActiveTab('commits')}
              className={`whitespace-nowrap py-4 px-3 border-b-2 font-medium text-sm ${
                activeTab === 'commits' 
                  ? 'border-indigo-500 text-indigo-600' 
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <svg className="h-5 w-5 inline mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
              </svg>
              Commits
              <span className="ml-1 px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-900">{commits.length}</span>
            </button>
            
            <button 
              onClick={() => setActiveTab('branches')}
              className={`whitespace-nowrap py-4 px-3 border-b-2 font-medium text-sm ${
                activeTab === 'branches' 
                  ? 'border-indigo-500 text-indigo-600' 
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <svg className="h-5 w-5 inline mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
              Branches
              <span className="ml-1 px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-900">{branches.length}</span>
            </button>
            
            <button 
              onClick={() => setActiveTab('wiki')}
              className={`whitespace-nowrap py-4 px-3 border-b-2 font-medium text-sm ${
                activeTab === 'wiki' 
                  ? 'border-indigo-500 text-indigo-600' 
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <svg className="h-5 w-5 inline mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              Wiki
            </button>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        {/* Content for active tab */}
        {activeTab === 'code' && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            {/* Branch selector and path breadcrumbs */}
            <div className="border-b border-gray-200 bg-gray-50 p-4 flex flex-col sm:flex-row justify-between items-start sm:items-center">
              <div className="flex items-center mb-4 sm:mb-0">
                <div className="relative">
                  <div className="flex items-center">
                    <svg className="h-5 w-5 text-gray-500 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z" />
                    </svg>
                    <select
                      className="appearance-none bg-transparent pr-8 py-1 pl-1 border border-gray-300 rounded-md text-sm font-medium text-gray-700 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
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
                </div>
                
                <div className="ml-4 text-sm bg-gray-100 rounded-md overflow-hidden">
                  <button 
                    onClick={() => setCurrentPath('')}
                    className={`px-3 py-1 ${currentPath ? 'text-indigo-600 hover:bg-gray-200' : 'bg-white text-gray-800 shadow-sm'}`}
                  >
                    {projectName}
                  </button>
                  
                  {currentPath && currentPath.split('/').filter(Boolean).map((part, index, parts) => (
                    <React.Fragment key={index}>
                      <span className="text-gray-400 px-1">/</span>
                      <button 
                        onClick={() => setCurrentPath(parts.slice(0, index + 1).join('/'))}
                        className={`px-2 py-1 ${index === parts.length - 1 ? 'bg-white text-gray-800 shadow-sm' : 'text-indigo-600 hover:bg-gray-200'}`}
                      >
                        {part}
                      </button>
                    </React.Fragment>
                  ))}
                </div>
              </div>

              <div className="flex items-center space-x-2 w-full sm:w-auto">
                <div className="relative flex-grow sm:flex-grow-0">
                  <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <svg className="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                  </div>
                  <input
                    type="search"
                    className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 text-sm focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500"
                    placeholder="Find file"
                  />
                </div>
                
                <button className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                  <svg className="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  Add file
                </button>
              </div>
            </div>
            
            {/* Files list */}
            {files.length === 0 ? (
              <div className="py-12 flex flex-col items-center justify-center text-center px-6">
                <svg className="h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
                </svg>
                <h3 className="text-lg font-medium text-gray-900 mb-1">This project is empty</h3>
                <p className="text-gray-500 mb-4 max-w-md">There are no files in this directory. You can add files using the button above.</p>
                <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                  Upload files
                </button>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead>
                    <tr>
                      <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Name
                      </th>
                      <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Last commit
                      </th>
                      <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Age
                      </th>
                      <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Size
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {currentPath && (
                      <tr className="hover:bg-gray-50" onClick={() => navigateToDirectory('..')}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                          <div className="flex items-center">
                            <svg className="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                            </svg>
                            <span className="text-indigo-600 hover:text-indigo-900">..</span>
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500"></td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500"></td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500"></td>
                      </tr>
                    )}
                    
                    {files.map((file, index) => (
                      <tr 
                        key={index}
                        className="hover:bg-gray-50"
                        onClick={() => file.type === 'directory' ? navigateToDirectory(file.name) : null}
                      >
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                          <div className="flex items-center">
                            {getFileIcon(file.name, file.type)}
                            <span className={file.type === 'directory' ? 'text-indigo-600 hover:text-indigo-900 cursor-pointer' : 'text-gray-900'}>
                              {file.name}
                            </span>
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {file.last_commit 
                            ? <span className="truncate block max-w-xs">{file.last_commit}</span>
                            : '-'}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {file.last_commit ? formatDate(file.last_commit) : '-'}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {file.type === 'file' ? formatFileSize(file.size) : '-'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            
            {/* README display */}
            {readmeContent && currentPath === '' && (
              <div className="border-t border-gray-200 p-6">
                <div className="flex items-center mb-4">
                  <svg className="h-5 w-5 text-gray-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  <h2 className="text-lg font-medium text-gray-900">README.md</h2>
                </div>
                <div className="prose prose-indigo max-w-none">
                  <div dangerouslySetInnerHTML={{ __html: readmeContent }} />
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'commits' && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <div className="p-4 border-b border-gray-200 bg-gray-50">
              <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center">
                <h2 className="text-lg font-medium text-gray-900">Commit History</h2>
                <div className="mt-2 sm:mt-0 flex items-center">
                  <div className="relative mr-3">
                    <div className="flex items-center">
                      <svg className="h-5 w-5 text-gray-500 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z" />
                      </svg>
                      <select
                        className="appearance-none bg-transparent pr-8 py-1 pl-1 border border-gray-300 rounded-md text-sm font-medium text-gray-700 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
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
                  </div>
                  
                  <button className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                    <svg className="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4h13M3 8h9m-9 4h6m4 0l4-4m0 0l4 4m-4-4v12" />
                    </svg>
                    Newest first
                  </button>
                </div>
              </div>
            </div>
            
            {commits.length === 0 ? (
              <div className="py-12 flex flex-col items-center justify-center text-center px-6">
                <svg className="h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
                </svg>
                <h3 className="text-lg font-medium text-gray-900 mb-1">No commits yet</h3>
                <p className="text-gray-500 mb-4 max-w-md">This branch doesn't have any commits yet. Create or push commits to get started.</p>
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {commits.map((commit, index) => (
                  <li key={index} className="px-6 py-4 hover:bg-gray-50">
                    <div className="flex flex-col sm:flex-row sm:items-start">
                      <div className="flex-grow">
                        <div className="flex items-center">
                          <div className="h-10 w-10 rounded-full bg-gray-200 flex-shrink-0 mr-3 overflow-hidden">
                            <svg className="h-full w-full text-gray-500" fill="currentColor" viewBox="0 0 24 24">
                              <path d="M24 20.993V24H0v-2.996A14.977 14.977 0 0112.004 15c4.904 0 9.26 2.354 11.996 5.993zM16.002 8.999a4 4 0 11-8 0 4 4 0 018 0z" />
                            </svg>
                          </div>
                          <div>
                            <h4 className="text-base font-medium text-gray-900">{commit.message}</h4>
                            <p className="text-sm text-gray-500">
                              <span className="font-medium">{commit.author}</span> committed {formatDate(commit.date)}
                            </p>
                          </div>
                        </div>
                      </div>
                      
                      <div className="mt-2 sm:mt-0 flex items-center">
                        <div className="bg-gray-100 rounded-md px-3 py-1 text-sm font-mono text-gray-800">
                          {commit.hash.substring(0, 7)}
                        </div>
                        <button className="ml-2 text-gray-400 hover:text-gray-500">
                          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                          </svg>
                        </button>
                        <button className="ml-2 text-gray-400 hover:text-gray-500">
                          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                          </svg>
                        </button>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {activeTab === 'branches' && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <div className="p-4 border-b border-gray-200 bg-gray-50">
              <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center">
                <h2 className="text-lg font-medium text-gray-900">Branches</h2>
                <button className="mt-2 sm:mt-0 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                  <svg className="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  New branch
                </button>
              </div>
            </div>
            
            {branches.length === 0 ? (
              <div className="py-12 flex flex-col items-center justify-center text-center px-6">
                <svg className="h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z" />
                </svg>
                <h3 className="text-lg font-medium text-gray-900 mb-1">No branches found</h3>
                <p className="text-gray-500 mb-4 max-w-md">This repository doesn't have any branches yet. Create the first branch to get started.</p>
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {branches.map((branch, index) => (
                  <li key={index} className="px-6 py-4 hover:bg-gray-50">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center">
                        <svg className="h-5 w-5 text-gray-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                        </svg>
                        <span className="text-lg font-medium text-gray-900">{branch}</span>
                        {branch === activeBranch && (
                          <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-md text-sm font-medium bg-green-100 text-green-800">
                            Default
                          </span>
                        )}
                      </div>
                      
                      <div className="flex items-center space-x-2">
                        <button 
                          onClick={() => handleBranchChange(branch)}
                          className={`px-3 py-1 text-sm rounded-md ${
                            activeBranch === branch 
                              ? 'bg-gray-100 text-gray-500 cursor-not-allowed' 
                              : 'text-indigo-600 hover:bg-indigo-50 hover:text-indigo-800'
                          }`}
                          disabled={activeBranch === branch}
                        >
                          {activeBranch === branch ? 'Current' : 'Switch'}
                        </button>
                        
                        <div className="relative" onClick={(e) => e.stopPropagation()}>
                          <button className="text-gray-400 hover:text-gray-500 focus:outline-none">
                            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                            </svg>
                          </button>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
        
        {activeTab === 'issues' && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <div className="p-4 border-b border-gray-200 bg-gray-50">
              <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center">
                <h2 className="text-lg font-medium text-gray-900">Issues</h2>
                <button className="mt-2 sm:mt-0 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                  <svg className="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  New issue
                </button>
              </div>
            </div>
            
            <div className="py-12 flex flex-col items-center justify-center text-center px-6">
              <svg className="h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <h3 className="text-lg font-medium text-gray-900 mb-1">No issues yet</h3>
              <p className="text-gray-500 mb-4 max-w-md">Issues are used to track todos, bugs, feature requests, and more.</p>
              <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                Create new issue
              </button>
            </div>
          </div>
        )}
        
        {activeTab === 'wiki' && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <div className="p-4 border-b border-gray-200 bg-gray-50">
              <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center">
                <h2 className="text-lg font-medium text-gray-900">Project Wiki</h2>
                <button className="mt-2 sm:mt-0 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                  <svg className="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                  Create wiki
                </button>
              </div>
            </div>
            
            <div className="py-12 flex flex-col items-center justify-center text-center px-6">
              <svg className="h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <h3 className="text-lg font-medium text-gray-900 mb-1">Welcome to the project wiki</h3>
              <p className="text-gray-500 mb-4 max-w-md">Wikis are a place to share documentation and information about your project with team members and contributors.</p>
              <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                Create the first page
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default RepoPage;
