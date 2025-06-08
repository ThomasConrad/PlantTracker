import { Component } from 'solid-js';
import { A } from '@solidjs/router';
import type { Plant } from '@/types';
import { calculateDaysUntil, isOverdue, formatRelativeTime } from '@/utils/date';

interface PlantCardProps {
  plant: Plant;
}

export const PlantCard: Component<PlantCardProps> = (props) => {
  const wateringDays = () => calculateDaysUntil(props.plant.lastWatered, props.plant.wateringIntervalDays);
  const fertilizingDays = () => calculateDaysUntil(props.plant.lastFertilized, props.plant.fertilizingIntervalDays);
  
  const wateringOverdue = () => isOverdue(props.plant.lastWatered, props.plant.wateringIntervalDays);
  const fertilizingOverdue = () => isOverdue(props.plant.lastFertilized, props.plant.fertilizingIntervalDays);

  const getStatusBadge = (days: number, overdue: boolean) => {
    if (overdue) return 'badge-red';
    if (days <= 1) return 'badge-yellow';
    return 'badge-green';
  };

  const getStatusText = (days: number, overdue: boolean, type: string) => {
    if (overdue) return `${type} overdue`;
    if (days === 0) return `${type} today`;
    if (days === 1) return `${type} tomorrow`;
    return `${type} in ${days} days`;
  };

  return (
    <A href={`/plants/${props.plant.id}`} class="plant-card">
      <div class="plant-thumbnail">
        <div class="h-full w-full bg-gradient-to-br from-primary-100 to-primary-200 flex items-center justify-center">
          <svg class="h-16 w-16 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width={1.5}
              d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
            />
          </svg>
        </div>
      </div>
      
      <div class="card-body">
        <div class="space-y-3">
          <div>
            <h3 class="text-lg font-medium text-gray-900 truncate">{props.plant.name}</h3>
            <p class="text-sm text-gray-500 italic">{props.plant.genus}</p>
          </div>
          
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-sm text-gray-600">Watering</span>
              <span class={`badge ${getStatusBadge(wateringDays(), wateringOverdue())}`}>
                {getStatusText(wateringDays(), wateringOverdue(), 'Water')}
              </span>
            </div>
            
            <div class="flex items-center justify-between">
              <span class="text-sm text-gray-600">Fertilizing</span>
              <span class={`badge ${getStatusBadge(fertilizingDays(), fertilizingOverdue())}`}>
                {getStatusText(fertilizingDays(), fertilizingOverdue(), 'Fertilize')}
              </span>
            </div>
          </div>
          
          <div class="pt-2 border-t border-gray-200">
            <p class="text-xs text-gray-500">
              Last activity: {props.plant.lastWatered || props.plant.lastFertilized 
                ? formatRelativeTime(
                    new Date(Math.max(
                      new Date(props.plant.lastWatered || 0).getTime(),
                      new Date(props.plant.lastFertilized || 0).getTime()
                    ))
                  )
                : 'Never'
              }
            </p>
          </div>
        </div>
      </div>
    </A>
  );
};