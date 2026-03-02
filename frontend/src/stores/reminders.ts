import { createSignal } from 'solid-js';
import { apiClient } from '@/api/client';

export interface ReminderPreferences {
  enabled: boolean;
  reminderTime: string;
  timezone: string;
  browserNotificationsEnabled: boolean;
}

export interface DueReminder {
  plantId: string;
  plantName: string;
  reminderType: 'watering' | 'fertilizing' | string;
  dueAt: string;
  dueDate: string;
  daysOverdue: number;
  alreadySent: boolean;
}

interface DueRemindersResponse {
  reminders: DueReminder[];
  totalDue: number;
  unsentCount: number;
}

const [dueCount, setDueCount] = createSignal(0);
const [loading, setLoading] = createSignal(false);
let pollHandle: number | null = null;

function hasNotificationSupport() {
  return typeof window !== 'undefined' && 'Notification' in window;
}

function nowLocalHHMM() {
  const now = new Date();
  const hh = String(now.getHours()).padStart(2, '0');
  const mm = String(now.getMinutes()).padStart(2, '0');
  return `${hh}:${mm}`;
}

function isAfterReminderTime(reminderTime: string) {
  return nowLocalHHMM() >= reminderTime;
}

const remindersStore = {
  get dueCount() {
    return dueCount();
  },
  get loading() {
    return loading();
  },

  async loadDueCount(): Promise<void> {
    try {
      setLoading(true);
      const response = await apiClient.getDueReminders();
      setDueCount(response.unsentCount);
    } catch (err) {
      console.error('Failed loading due reminders:', err);
    } finally {
      setLoading(false);
    }
  },

  async requestNotificationPermission(): Promise<'granted' | 'denied' | 'default' | 'unsupported'> {
    if (!hasNotificationSupport()) return 'unsupported';
    if (Notification.permission === 'granted') return 'granted';
    return Notification.requestPermission();
  },

  async triggerDueNotificationsIfReady(): Promise<void> {
    try {
      const prefs = await apiClient.getReminderPreferences();
      if (!prefs.enabled || !prefs.browserNotificationsEnabled) return;
      if (!isAfterReminderTime(prefs.reminderTime)) return;
      if (!hasNotificationSupport() || Notification.permission !== 'granted') return;

      const dispatched = await apiClient.dispatchDueReminders();
      for (const reminder of dispatched.reminders) {
        const title = `${reminder.reminderType === 'watering' ? 'Water' : 'Fertilize'} ${reminder.plantName}`;
        const body =
          reminder.daysOverdue > 0
            ? `${reminder.daysOverdue} day(s) overdue`
            : 'Due today';
        new Notification(title, { body, tag: `${reminder.plantId}-${reminder.reminderType}-${reminder.dueDate}` });
      }

      await this.loadDueCount();
    } catch (err) {
      console.error('Failed triggering due notifications:', err);
    }
  },

  async refreshAndNotify(): Promise<void> {
    await this.loadDueCount();
    await this.triggerDueNotificationsIfReady();
  },

  startPolling(intervalMs = 5 * 60 * 1000): void {
    if (pollHandle !== null) return;
    void this.refreshAndNotify();
    pollHandle = window.setInterval(() => {
      void this.refreshAndNotify();
    }, intervalMs);
  },

  stopPolling(): void {
    if (pollHandle !== null) {
      window.clearInterval(pollHandle);
      pollHandle = null;
    }
  },
};

export type { DueRemindersResponse };
export { remindersStore };
