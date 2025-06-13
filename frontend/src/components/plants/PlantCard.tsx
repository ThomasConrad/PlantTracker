import { Component, Show } from 'solid-js';
import { A } from '@solidjs/router';
import type { Plant } from '@/types';

interface PlantCardProps {
  plant: Plant;
  mobileColumns?: number;
}

export const PlantCard: Component<PlantCardProps> = (props) => {
  const getTextClasses = () => {
    const cols = props.mobileColumns || 1;
    
    if (cols === 1) {
      return {
        title: "text-white font-semibold text-xl leading-tight mb-1 drop-shadow-sm",
        subtitle: "text-white/90 text-base italic drop-shadow-sm",
        padding: "p-4"
      };
    } else if (cols === 2) {
      return {
        title: "text-white font-semibold text-lg leading-tight mb-1 drop-shadow-sm",
        subtitle: "text-white/90 text-sm italic drop-shadow-sm",
        padding: "p-3"
      };
    } else if (cols === 3) {
      return {
        title: "text-white font-semibold text-base leading-tight mb-0.5 drop-shadow-sm",
        subtitle: "text-white/90 text-xs italic drop-shadow-sm",
        padding: "p-2.5"
      };
    } else if (cols === 4) {
      return {
        title: "text-white font-semibold text-sm leading-tight mb-0.5 drop-shadow-sm",
        subtitle: "text-white/90 text-xs italic drop-shadow-sm",
        padding: "p-2"
      };
    } else {
      // For 5+ columns, use minimal text
      return {
        title: "text-white font-semibold text-xs leading-tight mb-0 drop-shadow-sm truncate",
        subtitle: "text-white/90 text-xs italic drop-shadow-sm truncate",
        padding: "p-1.5"
      };
    }
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
        <div class={`absolute bottom-0 left-0 right-0 ${getTextClasses().padding}`}>
          <h3 class={getTextClasses().title}>
            {props.plant.name}
          </h3>
          <p class={getTextClasses().subtitle}>
            {props.plant.genus}
          </p>
        </div>
      </div>
    </A>
  );
};