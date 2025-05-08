import React, { useEffect, useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import FileDiff from '../components/FileDiff';

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

interface Comment {
  id: number;
  pull_request_id: number;
  author_id: number;
  content: string;
  created_at: string;
}

interface FileDiffInfo {
  filename: string;
  diffLines: {
    content: string;
    type: 'added' | 'removed' | 'unchanged';
    oldLineNumber: number | null;
    newLineNumber: number | null;
  }[];
}

interface PullRequestDetails {
  pull_request: PullRequest;
  comments: Comment[];
  diffs: FileDiffInfo[];
}

interface ApiResponse {
  success: boolean;
  message: string | null;
  data: PullRequestDetails | null;
}

const PullRequestPage: React.FC = () => {
  const { repoName, pullId } = useParams<{ repoName: string; pullId: string }>();
  const navigate = useNavigate();
  const [prDetails, setPrDetails] = useState<PullRequestDetails | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [newComment, setNewComment] = useState<string>('');
  const [statusUpdate, setStatusUpdate] = useState<string>('');
  const [expandedDiffs, setExpandedDiffs] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const fetchPullRequestDetails = async () => {
      try {
        const response = await fetch(`http://localhost:8000/api/repos/${repoName}/pulls/${pullId}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
          }
        });

        const data: ApiResponse = await response.json();
        
        if (data.success && data.data) {
          setPrDetails(data.data);
        } else {
          setError(data.message || 'Failed to fetch pull request details');
        }
      } catch (err) {
        setError('Error connecting to the server');
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    if (repoName && pullId) {
      fetchPullRequestDetails();
    }
  }, [repoName, pullId]);
  
  // Initialize expanded state for all diffs when they're loaded
  useEffect(() => {
    if (prDetails && prDetails.diffs) {
      const initialExpandedState: Record<string, boolean> = {};
      prDetails.diffs.forEach((diff) => {
        initialExpandedState[diff.filename] = true; // Default to expanded
      });
      setExpandedDiffs(initialExpandedState);
    }
  }, [prDetails]);

  const handleCommentSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!newComment.trim()) return;
    
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${repoName}/pulls/${pullId}/comments`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
        },
        body: JSON.stringify({
          content: newComment
        })
      });
      
      const data = await response.json();
      
      if (data.success) {
        // Refresh PR details to show the new comment
        const prResponse = await fetch(`http://localhost:8000/api/repos/${repoName}/pulls/${pullId}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
          }
        });
        
        const prData: ApiResponse = await prResponse.json();
        
        if (prData.success && prData.data) {
          setPrDetails(prData.data);
        }
        
        setNewComment('');
      } else {
        setError(data.message || 'Failed to add comment');
      }
    } catch (err) {
      setError('Error adding comment');
      console.error(err);
    }
  };

  const handleStatusUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!statusUpdate) return;
    
    try {
      const response = await fetch(`http://localhost:8000/api/repos/${repoName}/pulls/${pullId}/status`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
        },
        body: JSON.stringify({
          status: statusUpdate
        })
      });
      
      const data = await response.json();
      
      if (data.success) {
        // Refresh PR details to show the updated status
        const prResponse = await fetch(`http://localhost:8000/api/repos/${repoName}/pulls/${pullId}`, {
          headers: {
            'Authorization': `Basic ${btoa('Kazilsky:password123')}` // Replace with actual auth
          }
        });
        
        const prData: ApiResponse = await prResponse.json();
        
        if (prData.success && prData.data) {
          setPrDetails(prData.data);
        }
        
        setStatusUpdate('');
      } else {
        setError(data.message || 'Failed to update status');
      }
    } catch (err) {
      setError('Error updating status');
      console.error(err);
    }
  };

  const renderStatusBadge = (status: string) => {
    const statusClasses = {
      open: 'bg-green-100 text-green-800',
      closed: 'bg-red-100 text-red-800',
      merged: 'bg-purple-100 text-purple-800'
    };
    
    const statusClass = statusClasses[status as keyof typeof statusClasses] || 'bg-gray-100 text-gray-800';
    
    return (
      <span className={`px-2 py-1 text-xs rounded-full ${statusClass}`}>
        {status}
      </span>
    );
  };

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

  if (!prDetails) {
    return (
      <div className="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded relative" role="alert">
        <strong className="font-bold">Warning:</strong>
        <span className="block sm:inline"> Pull request not found</span>
      </div>
    );
  }

  const { pull_request: pr, comments, diffs } = prDetails;

  const handleToggleDiff = (filename: string) => {
    setExpandedDiffs(prev => ({
      ...prev,
      [filename]: !prev[filename]
    }));
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-4">
        <Link 
          to={`/repo/${repoName}`}
          className="text-blue-600 hover:text-blue-800 flex items-center"
        >
          <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
          Back to repository
        </Link>
      </div>
      
      <div className="bg-white rounded-lg shadow-md overflow-hidden mb-6">
        <div className="p-6 border-b">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold">{pr.title}</h1>
            {renderStatusBadge(pr.status)}
          </div>
          
          <div className="mt-2 text-sm text-gray-600">
            <span>#{pr.id}</span>
            <span className="mx-2">•</span>
            <span>{pr.source_branch} → {pr.target_branch}</span>
            <span className="mx-2">•</span>
            <span>Created: {new Date(pr.created_at).toLocaleDateString()}</span>
          </div>
          
          {pr.description && (
            <div className="mt-4 p-4 bg-gray-50 rounded-md">
              <p className="whitespace-pre-wrap">{pr.description}</p>
            </div>
          )}
          
          {pr.status === 'open' && (
            <div className="mt-6">
              <form onSubmit={handleStatusUpdate} className="flex items-center">
                <select
                  value={statusUpdate}
                  onChange={(e) => setStatusUpdate(e.target.value)}
                  className="border rounded-l-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                >
                  <option value="">Update status...</option>
                  <option value="closed">Close</option>
                  <option value="merged">Merge</option>
                </select>
                <button
                  type="submit"
                  className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-r-md"
                  disabled={!statusUpdate}
                >
                  Update
                </button>
              </form>
            </div>
          )}
        </div>
        
        <div className="p-6">
          <h2 className="text-lg font-medium mb-4">Comments ({comments.length})</h2>
          
          {comments.length === 0 ? (
            <div className="text-center py-8 bg-gray-50 rounded-md">
              <p className="text-gray-600">No comments yet</p>
            </div>
          ) : (
            <div className="space-y-4">
              {comments.map((comment) => (
                <div key={comment.id} className="p-4 border rounded-md">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium">User #{comment.author_id}</span>
                    <span className="text-sm text-gray-500">
                      {new Date(comment.created_at).toLocaleString()}
                    </span>
                  </div>
                  <p className="whitespace-pre-wrap">{comment.content}</p>
                </div>
              ))}
            </div>
          )}
          
          <div className="mt-6">
            <h3 className="text-md font-medium mb-2">Add a comment</h3>
            <form onSubmit={handleCommentSubmit}>
              <textarea
                value={newComment}
                onChange={(e) => setNewComment(e.target.value)}
                className="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                rows={4}
                placeholder="Write your comment here..."
                required
              />
              <div className="mt-2 flex justify-end">
                <button
                  type="submit"
                  className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md"
                  disabled={!newComment.trim()}
                >
                  Comment
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
      
      {/* File Diffs Section */}
      <div className="bg-white rounded-lg shadow-md overflow-hidden">
        <div className="p-6 border-b">
          <h2 className="text-lg font-medium">Changes</h2>
          <div className="text-sm text-gray-600 mt-1">
            Showing {diffs?.length || 0} changed files
          </div>
        </div>
        
        <div className="p-6">
          {diffs && diffs.length > 0 ? (
            <div>
              {diffs.map((diff) => (
                <FileDiff
                  key={diff.filename}
                  filename={diff.filename}
                  diffLines={diff.diffLines}
                  expanded={expandedDiffs[diff.filename] || false}
                  onToggleExpand={() => handleToggleDiff(diff.filename)}
                />
              ))}
            </div>
          ) : (
            <div className="text-center py-8 bg-gray-50 rounded-md">
              <p className="text-gray-600">No file changes found</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PullRequestPage;
