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
    <A href={`/plants/${props.plant.id}`} class="plant-card-full-image">
      <div class="relative aspect-[4/5] overflow-hidden rounded-lg">
        <Show 
          when={props.plant.thumbnailUrl} 
          fallback={
            <div class="h-full w-full bg-gradient-to-br from-primary-100 to-primary-200 flex items-center justify-center">
              <svg class="h-16 w-16 text-primary-600 opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
        
        {/* Gradient overlay for better text contrast */}
        <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-black/20 to-transparent" />
        
        {/* Plant name and genus overlay */}
        <div class="absolute bottom-0 left-0 right-0 p-4">
          <h3 class="text-white font-semibold text-lg leading-tight mb-1 drop-shadow-sm">
            {props.plant.name}
          </h3>
          <p class="text-white/90 text-sm italic drop-shadow-sm">
            {props.plant.genus}
          </p>
        </div>
      </div>
    </A>
  );
};