// components/ProjectCard.tsx
import React from 'react';
import { Link } from 'react-router-dom';
import { FiFolder, FiGitBranch } from 'react-icons/fi';

interface ProjectCardProps {
  project: {
    id: number;
    name: string;
    description: string | null;
    repositories_count: number;
    created_at: string;
  };
}

const ProjectCard: React.FC<ProjectCardProps> = ({ project }) => {
  return (
    <div className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-all duration-200">
      <Link to={`/project/${project.id}`} className="block">
        <div className="px-4 py-5 sm:p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0 bg-green-500 rounded-md p-2">
              <FiFolder className="h-6 w-6 text-white" />
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-medium text-gray-900">{project.name}</h3>
              <p className="text-sm text-gray-500 mt-1 truncate">
                {project.description || 'No description provided'}
              </p>
            </div>
          </div>
          <div className="mt-6 flex justify-between items-center">
            <span className="inline-flex items-center text-sm text-gray-500">
              <FiGitBranch className="mr-1 text-gray-400" />
              {project.repositories_count} repositories
            </span>
            <span className="inline-flex items-center text-xs text-gray-500">
              Created: {new Date(project.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
      </Link>
    </div>
  );
};

export default ProjectCard;