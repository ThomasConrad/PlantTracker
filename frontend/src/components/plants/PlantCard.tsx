import { Component, Show } from 'solid-js';
import { A } from '@solidjs/router';
import type { Plant } from '@/types';
import { calculateDaysUntil, isOverdue, formatRelativeTime } from '@/utils/date';

interface PlantCardProps {
  plant: Plant;
}

export const PlantCard: Component<PlantCardProps> = (props) => {
  const wateringDays = () => calculateDaysUntil(props.plant.lastWatered ?? null, props.plant.wateringIntervalDays);
  const fertilizingDays = () => calculateDaysUntil(props.plant.lastFertilized ?? null, props.plant.fertilizingIntervalDays);
  
  const wateringOverdue = () => isOverdue(props.plant.lastWatered ?? null, props.plant.wateringIntervalDays);
  const fertilizingOverdue = () => isOverdue(props.plant.lastFertilized ?? null, props.plant.fertilizingIntervalDays);

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
        <Show 
          when={props.plant.thumbnailUrl} 
          fallback={
            <div class="h-full w-full bg-gradient-to-br from-primary-100 to-primary-200 flex items-center justify-center">
              <svg class="h-12 w-12 sm:h-16 sm:w-16 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={1.5}
                  d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                />
              </svg>
            </div>
          }
        >
          <img 
            src={props.plant.thumbnailUrl!} 
            alt={`${props.plant.name} thumbnail`}
            class="h-full w-full object-cover"
            loading="lazy"
            onError={(e) => {
              // If thumbnail fails to load, it might still be processing
              // Try again in a few seconds
              const img = e.currentTarget;
              setTimeout(() => {
                img.src = props.plant.thumbnailUrl! + `?retry=${Date.now()}`;
              }, 3000);
            }}
          />
        </Show>
      </div>
      
      <div class="plant-card-body">
        <div class="space-y-4 sm:space-y-3">
          <div>
            <h3 class="plant-card-title">{props.plant.name}</h3>
            <p class="plant-card-subtitle">{props.plant.genus}</p>
          </div>
          
          <div class="space-y-3 sm:space-y-2">
            <div class="plant-card-status">
              <span class="text-sm text-gray-600 font-medium">Watering</span>
              <span class={`badge-mobile ${getStatusBadge(wateringDays(), wateringOverdue())}`}>
                {getStatusText(wateringDays(), wateringOverdue(), 'Water')}
              </span>
            </div>
            
            <div class="plant-card-status">
              <span class="text-sm text-gray-600 font-medium">Fertilizing</span>
              <span class={`badge-mobile ${getStatusBadge(fertilizingDays(), fertilizingOverdue())}`}>
                {getStatusText(fertilizingDays(), fertilizingOverdue(), 'Fertilize')}
              </span>
            </div>
          </div>
          
          <div class="plant-card-footer">
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