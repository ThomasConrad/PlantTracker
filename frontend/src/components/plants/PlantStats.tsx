import { Component } from 'solid-js';
import type { Plant } from '@/types';
import { calculateDaysUntil, isOverdue } from '@/utils/date';

interface PlantStatsProps {
  plant: Plant;
}

export const PlantStats: Component<PlantStatsProps> = (props) => {
  const wateringDays = () => calculateDaysUntil(props.plant.lastWatered ?? null, props.plant.wateringIntervalDays);
  const fertilizingDays = () => calculateDaysUntil(props.plant.lastFertilized ?? null, props.plant.fertilizingIntervalDays);
  
  const wateringOverdue = () => isOverdue(props.plant.lastWatered ?? null, props.plant.wateringIntervalDays);
  const fertilizingOverdue = () => isOverdue(props.plant.lastFertilized ?? null, props.plant.fertilizingIntervalDays);

  const getStatusColor = (days: number, overdue: boolean) => {
    if (overdue) return 'text-red-600 bg-red-50';
    if (days <= 1) return 'text-yellow-600 bg-yellow-50';
    return 'text-green-600 bg-green-50';
  };

  const getStatusText = (days: number, overdue: boolean, type: string) => {
    if (overdue) return `${type} overdue`;
    if (days === 0) return `${type} today`;
    if (days === 1) return `${type} tomorrow`;
    return `${type} in ${days} days`;
  };

  return (
    <div class="card">
      <div class="card-header">
        <h3 class="text-lg font-medium text-gray-900">Care Status</h3>
      </div>
      <div class="card-body">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
          <div class={`p-4 rounded-lg ${getStatusColor(wateringDays(), wateringOverdue())}`}>
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={2}
                    d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707"
                  />
                </svg>
              </div>
              <div class="ml-4">
                <h4 class="text-sm font-medium">Watering</h4>
                <p class="text-lg font-semibold">
                  {getStatusText(wateringDays(), wateringOverdue(), 'Water')}
                </p>
              </div>
            </div>
          </div>

          <div class={`p-4 rounded-lg ${getStatusColor(fertilizingDays(), fertilizingOverdue())}`}>
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={2}
                    d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
                  />
                </svg>
              </div>
              <div class="ml-4">
                <h4 class="text-sm font-medium">Fertilizing</h4>
                <p class="text-lg font-semibold">
                  {getStatusText(fertilizingDays(), fertilizingOverdue(), 'Fertilize')}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};