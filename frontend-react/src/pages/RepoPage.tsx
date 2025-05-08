import React, { useEffect, useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { Tab } from '@headlessui/react';
import CodeEditor from '../components/CodeEditor';

interface Repository {
  id: number;
  name: string;
  description: string | null;
  owner_id: number;
  is_public: boolean;
  created_at: string;
}

interface PullRequest {
  id: number;
  title: string;
  description: string | null;
  repository_id: number;
  source_branch: string;
  target_branch: string;
  author_id: number;
  status: string;
  created_at: string;
  updated_at: string;
}

interface RepoDetails {
  repo: Repository;
  branches: string[];
  pull_requests: PullRequest[];
}

interface ApiResponse {
  success: boolean;
  message: string | null;
  data: RepoDetails | null;
}

interface FileContent {
  path: string;
  content: string;
  isDirectory: boolean;
}

const RepoPage: React.FC = () => {
  const { repoName } = useParams<{ repoName: string }>();
  const navigate = useNavigate();
  const [repoDetails, setRepoDetails] = useState<RepoDetails | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPath, setCurrentPath] = useState<string>('');
  const [fileContent, setFileContent] = useState<FileContent[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileData, setFileData] = useState<string>('');
  const [newPullRequest, setNewPullRequest] = useState({
    title: '',
    description: '',
    sourceBranch: '',
    targetBranch: 'main'
  });
  const [showNewPrForm, setShowNewPrForm] = useState(false);

  useEffect(() => {
    const fetchRepoDetails = async () => {
      try {
        const response = await fetch(`http://localhost:8000/api/repos/${repoName}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
          }
        });

        const data: ApiResponse = await response.json();
        
        if (data.success && data.data) {
          setRepoDetails(data.data);
          // If there are branches, fetch the default branch content
          if (data.data.branches.length > 0) {
            fetchRepoContent(data.data.branches[0], '');
          }
        } else {
          setError(data.message || 'Failed to fetch repository details');
        }
      } catch (err) {
        setError('Error connecting to the server');
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    if (repoName) {
      fetchRepoDetails();
    }
  }, [repoName]);

  const fetchRepoContent = async (branch: string, path: string) => {
    try {
      setLoading(true);
      // This is a simplified example - in a real app, you'd fetch the actual file listing
      // from your backend API
      const mockFiles: FileContent[] = [
        { path: 'README.md', content: '', isDirectory: false },
        { path: 'src', content: '', isDirectory: true },
        { path: 'package.json', content: '', isDirectory: false },
      ];
      
      setFileContent(mockFiles);
      setCurrentPath(path);
      setLoading(false);
    } catch (err) {
      setError('Error fetching repository content');
      console.error(err);
      setLoading(false);
    }
  };

  const fetchFileContent = async (filePath: string) => {
    try {
      setLoading(true);
      // In a real app, you'd fetch the actual file content from your backend API
      const response = await fetch(`http://localhost:8000/git/${repoName}/file/${filePath}`, {
        headers: {
          'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
        }
      });
      
      if (response.ok) {
        const content = await response.text();
        setFileData(content);
        setSelectedFile(filePath);
      } else {
        setError('Failed to fetch file content');
      }
      setLoading(false);
    } catch (err) {
      setError('Error fetching file content');
      console.error(err);
      setLoading(false);
    }
  };

  const handleFileClick = (file: FileContent) => {
    if (file.isDirectory) {
      fetchRepoContent(repoDetails?.branches[0] || 'main', `${currentPath}/${file.path}`.replace(/^\//, ''));
    } else {
      fetchFileContent(`${currentPath}/${file.path}`.replace(/^\//, ''));
    }
  };

  const handleBreadcrumbClick = (path: string) => {
    fetchRepoContent(repoDetails?.branches[0] || 'main', path);
    setSelectedFile(null);
  };

  const handlePullRequestSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${repoName}/pulls`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
        },
        body: JSON.stringify({
          title: newPullRequest.title,
          description: newPullRequest.description,
          source_branch: newPullRequest.sourceBranch,
          target_branch: newPullRequest.targetBranch
        })
      });
      
      const data = await response.json();
      
      if (data.success) {
        // Refresh repo details to show the new PR
        const repoResponse = await fetch(`http://localhost:8000/api/repos/${repoName}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
          }
        });
        
        const repoData: ApiResponse = await repoResponse.json();
        
        if (repoData.success && repoData.data) {
          setRepoDetails(repoData.data);
        }
        
        setShowNewPrForm(false);
        setNewPullRequest({
          title: '',
          description: '',
          sourceBranch: '',
          targetBranch: 'main'
        });
      } else {
        setError(data.message || 'Failed to create pull request');
      }
    } catch (err) {
      setError('Error creating pull request');
      console.error(err);
    }
  };

  const renderBreadcrumbs = () => {
    const paths = currentPath.split('/').filter(Boolean);
    let fullPath = '';
    
    return (
      <div className="flex items-center text-sm text-gray-600 mb-4">
        <button 
          onClick={() => handleBreadcrumbClick('')}
          className="hover:text-blue-600"
        >
          root
        </button>
        {paths.map((path, index) => {
          fullPath += `/${path}`;
          return (
            <React.Fragment key={index}>
              <span className="mx-1">/</span>
              <button 
                onClick={() => handleBreadcrumbClick(fullPath)}
                className="hover:text-blue-600"
              >
                {path}
              </button>
            </React.Fragment>
          );
        })}
      </div>
    );
  };

  const renderFileExplorer = () => {
    return (
      <div className="border rounded-md overflow-hidden">
        {renderBreadcrumbs()}
        
        <div className="divide-y">
          {fileContent.map((file, index) => (
            <div 
              key={index}
              className="px-4 py-2 hover:bg-gray-50 cursor-pointer flex items-center"
              onClick={() => handleFileClick(file)}
            >
              {file.isDirectory ? (
                <svg className="w-5 h-5 mr-2 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" clipRule="evenodd" />
                </svg>
              ) : (
                <svg className="w-5 h-5 mr-2 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clipRule="evenodd" />
                </svg>
              )}
              <span>{file.path}</span>
            </div>
          ))}
          
          {fileContent.length === 0 && (
            <div className="px-4 py-8 text-center text-gray-500">
              This directory is empty
            </div>
          )}
        </div>
      </div>
    );
  };

  const renderFileViewer = () => {
    if (!selectedFile) return null;
    
    return (
      <div className="mt-4">
        <div className="bg-gray-100 px-4 py-2 text-sm font-mono border-t border-l border-r rounded-t-md">
          {selectedFile}
        </div>
        <CodeEditor 
          value={fileData}
          language={selectedFile}
          readOnly={true}
          height={400}
          theme="light"
        />
      </div>
    );
  };

  const renderPullRequests = () => {
    if (!repoDetails?.pull_requests || repoDetails.pull_requests.length === 0) {
      return (
        <div className="text-center py-8 bg-gray-50 rounded-md">
          <p className="text-gray-600">No pull requests found for this repository.</p>
          <button 
            onClick={() => setShowNewPrForm(true)}
            className="mt-4 bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded"
          >
            Create Pull Request
          </button>
        </div>
      );
    }
    
    return (
      <div>
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-medium">Pull Requests</h3>
          <button 
            onClick={() => setShowNewPrForm(true)}
            className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-1 px-3 rounded text-sm"
          >
            New Pull Request
          </button>
        </div>
        
        <div className="divide-y border rounded-md">
          {repoDetails.pull_requests.map((pr) => (
            <div key={pr.id} className="p-4 hover:bg-gray-50">
              <div className="flex items-center justify-between">
                <Link 
                  to={`/repo/${repoName}/pull/${pr.id}`} 
                  className="text-blue-600 hover:text-blue-800 font-medium"
                >
                  {pr.title}
                </Link>
                <span className={`px-2 py-1 text-xs rounded-full ${
                  pr.status === 'open' ? 'bg-green-100 text-green-800' : 
                  pr.status === 'closed' ? 'bg-red-100 text-red-800' : 
                  'bg-purple-100 text-purple-800'
                }`}>
                  {pr.status}
                </span>
              </div>
              <div className="mt-1 text-sm text-gray-600">
                <span>{pr.source_branch} → {pr.target_branch}</span>
                <span className="ml-4">Created: {new Date(pr.created_at).toLocaleDateString()}</span>
              </div>
              {pr.description && (
                <p className="mt-2 text-sm text-gray-700">{pr.description}</p>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderNewPullRequestForm = () => {
    if (!showNewPrForm) return null;
    
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white rounded-lg p-6 w-full max-w-lg">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-medium">Create New Pull Request</h3>
            <button 
              onClick={() => setShowNewPrForm(false)}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          
          <form onSubmit={handlePullRequestSubmit}>
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-1">Title</label>
              <input
                type="text"
                value={newPullRequest.title}
                onChange={(e) => setNewPullRequest({...newPullRequest, title: e.target.value})}
                className="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                required
              />
            </div>
            
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
              <textarea
                value={newPullRequest.description}
                onChange={(e) => setNewPullRequest({...newPullRequest, description: e.target.value})}
                className="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                rows={4}
              />
            </div>
            
            <div className="grid grid-cols-2 gap-4 mb-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Source Branch</label>
                <select
                  value={newPullRequest.sourceBranch}
                  onChange={(e) => setNewPullRequest({...newPullRequest, sourceBranch: e.target.value})}
                  className="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                >
                  <option value="">Select branch</option>
                  {repoDetails?.branches.map((branch) => (
                    <option key={branch} value={branch}>{branch}</option>
                  ))}
                </select>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Target Branch</label>
                <select
                  value={newPullRequest.targetBranch}
                  onChange={(e) => setNewPullRequest({...newPullRequest, targetBranch: e.target.value})}
                  className="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                >
                  <option value="">Select branch</option>
                  {repoDetails?.branches.map((branch) => (
                    <option key={branch} value={branch}>{branch}</option>
                  ))}
                </select>
              </div>
            </div>
            
            <div className="flex justify-end">
              <button
                type="button"
                onClick={() => setShowNewPrForm(false)}
                className="mr-2 px-4 py-2 border rounded-md text-gray-700 hover:bg-gray-100"
              >
                Cancel
              </button>
              <button
                type="submit"
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Create Pull Request
              </button>
            </div>
          </form>
        </div>
      </div>
    );
  };

  if (loading && !repoDetails) {
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

  if (!repoDetails) {
    return (
      <div className="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded relative" role="alert">
        <strong className="font-bold">Warning:</strong>
        <span className="block sm:inline"> Repository not found</span>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold">{repoDetails.repo.name}</h1>
          <div className="flex space-x-2">
            <button 
              onClick={() => navigate(`/repo/${repoName}/settings`)}
              className="px-3 py-1 border rounded-md hover:bg-gray-100"
            >
              Settings
            </button>
            <button className="px-3 py-1 bg-green-600 text-white rounded-md hover:bg-green-700">
              Clone
            </button>
          </div>
        </div>
        {repoDetails.repo.description && (
          <p className="text-gray-600 mt-2">{repoDetails.repo.description}</p>
        )}
      </div>

      <Tab.Group>
        <Tab.List className="flex border-b">
          <Tab className={({ selected }: { selected: boolean }) => 
            `px-4 py-2 font-medium text-sm focus:outline-none ${
              selected ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500 hover:text-gray-700'
            }`
          }>
            Code
          </Tab>
          <Tab className={({ selected }: { selected: boolean }) => 
            `px-4 py-2 font-medium text-sm focus:outline-none ${
              selected ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-500 hover:text-gray-700'
            }`
          }>
            Pull Requests
          </Tab>
        </Tab.List>
        <Tab.Panels className="mt-4">
          <Tab.Panel>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center">
                <select 
                  className="border rounded-md px-3 py-1 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  value={repoDetails.branches[0] || ''}
                  onChange={(e) => fetchRepoContent(e.target.value, currentPath)}
                >
                  {repoDetails.branches.map((branch) => (
                    <option key={branch} value={branch}>{branch}</option>
                  ))}
                </select>
              </div>
            </div>
            
            {renderFileExplorer()}
            {renderFileViewer()}
          </Tab.Panel>
          <Tab.Panel>
            {renderPullRequests()}
          </Tab.Panel>
        </Tab.Panels>
      </Tab.Group>
      
      {renderNewPullRequestForm()}
    </div>
  );
};

export default RepoPage;
