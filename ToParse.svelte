<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { ComponentProps } from "svelte";

  interface User {
    id: number;
    name: string;
    email: string;
    avatar: string;
    status: "online" | "offline" | "away";
    lastSeen: Date;
    roles: string[];
    metadata: Record<string, unknown>;
  }

  interface Message {
    id: string;
    content: string;
    author: User;
    timestamp: Date;
    attachments: Attachment[];
    reactions: Reaction[];
    thread?: Message[];
    edited: boolean;
    pinned: boolean;
  }

  interface Attachment {
    id: string;
    type: "image" | "video" | "file" | "audio";
    url: string;
    name: string;
    size: number;
    mimeType: string;
  }

  interface Reaction {
    emoji: string;
    users: User[];
    count: number;
  }

  interface Channel {
    id: string;
    name: string;
    description: string;
    type: "text" | "voice" | "announcement";
    members: User[];
    messages: Message[];
    settings: ChannelSettings;
  }

  interface ChannelSettings {
    slowMode: number;
    nsfw: boolean;
    archived: boolean;
    locked: boolean;
    permissions: Permission[];
  }

  interface Permission {
    role: string;
    allow: string[];
    deny: string[];
  }

  let {
    users = $bindable([]),
    channels = $bindable([]),
    currentUser,
    theme = "dark",
    locale = "en-US",
    onMessageSend,
    onUserClick,
    onChannelSelect,
    class: className = "",
    ...restProps
  }: {
    users?: User[];
    channels?: Channel[];
    currentUser: User;
    theme?: "light" | "dark" | "system";
    locale?: string;
    onMessageSend?: (message: Partial<Message>) => Promise<void>;
    onUserClick?: (user: User) => void;
    onChannelSelect?: (channel: Channel) => void;
    class?: string;
  } & ComponentProps<HTMLDivElement> = $props();

  const dispatch = createEventDispatcher<{
    message: Message;
    reaction: { messageId: string; emoji: string };
    typing: { userId: number; channelId: string };
    presence: { userId: number; status: User["status"] };
  }>();

  let selectedChannel = $state<Channel | null>(null);
  let messageInput = $state("");
  let isTyping = $state(false);
  let typingUsers = $state<User[]>([]);
  let searchQuery = $state("");
  let showEmojiPicker = $state(false);
  let showUserList = $state(true);
  let unreadCounts = $state<Record<string, number>>({});
  let scrollContainer = $state<HTMLElement | null>(null);
  let messageRefs = $state<Map<string, HTMLElement>>(new Map());

  const filteredUsers = $derived(
          users.filter(
                  (u) =>
                          u.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          u.email.toLowerCase().includes(searchQuery.toLowerCase()),
          ),
  );

  const sortedChannels = $derived(
          [...channels].sort((a, b) => {
            const aUnread = unreadCounts[a.id] || 0;
            const bUnread = unreadCounts[b.id] || 0;
            if (aUnread !== bUnread) return bUnread - aUnread;
            return a.name.localeCompare(b.name);
          }),
  );

  const onlineUsers = $derived(users.filter((u) => u.status === "online"));
  const offlineUsers = $derived(users.filter((u) => u.status === "offline"));
  const awayUsers = $derived(users.filter((u) => u.status === "away"));

  const currentMessages = $derived(selectedChannel?.messages ?? []);
  const pinnedMessages = $derived(currentMessages.filter((m) => m.pinned));
  const hasUnread = $derived(Object.values(unreadCounts).some((c) => c > 0));

  $effect(() => {
    if (selectedChannel) {
      unreadCounts[selectedChannel.id] = 0;
    }
  });

  $effect(() => {
    if (isTyping && selectedChannel) {
      const timeout = setTimeout(() => {
        isTyping = false;
      }, 3000);
      return () => clearTimeout(timeout);
    }
  });

  $effect(() => {
    if (scrollContainer && currentMessages.length > 0) {
      tick().then(() => {
        scrollContainer?.scrollTo({
          top: scrollContainer.scrollHeight,
          behavior: "smooth",
        });
      });
    }
  });

  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        showEmojiPicker = false;
      }
      if (e.ctrlKey && e.key === "k") {
        e.preventDefault();
        document
                .querySelector<HTMLInputElement>("#search-input")
                ?.focus();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });

  async function handleSendMessage() {
    if (!messageInput.trim() || !selectedChannel) return;

    const newMessage: Partial<Message> = {
      content: messageInput,
      author: currentUser,
      timestamp: new Date(),
      attachments: [],
      reactions: [],
      edited: false,
      pinned: false,
    };

    try {
      await onMessageSend?.(newMessage);
      dispatch("message", newMessage as Message);
      messageInput = "";
      isTyping = false;
    } catch (error) {
      console.error("Failed to send message:", error);
    }
  }

  function handleReaction(messageId: string, emoji: string) {
    dispatch("reaction", { messageId, emoji });
  }

  function handleChannelClick(channel: Channel) {
    selectedChannel = channel;
    onChannelSelect?.(channel);
  }

  function handleUserPresence(user: User, status: User["status"]) {
    dispatch("presence", { userId: user.id, status });
  }

  function formatTimestamp(date: Date): string {
    return new Intl.DateTimeFormat(locale, {
      hour: "numeric",
      minute: "numeric",
      hour12: true,
    }).format(date);
  }

  function formatDate(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) return "Today";
    if (days === 1) return "Yesterday";
    if (days < 7) return `${days} days ago`;

    return new Intl.DateTimeFormat(locale, {
      month: "short",
      day: "numeric",
      year: days > 365 ? "numeric" : undefined,
    }).format(date);
  }

  function getStatusColor(status: User["status"]): string {
    switch (status) {
      case "online":
        return "bg-green-500";
      case "away":
        return "bg-yellow-500";
      case "offline":
        return "bg-gray-500";
    }
  }

  function formatFileSize(bytes: number): string {
    const units = ["B", "KB", "MB", "GB"];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  function groupMessagesByDate(messages: Message[]): Map<string, Message[]> {
    const groups = new Map<string, Message[]>();

    for (const message of messages) {
      const dateKey = message.timestamp.toDateString();
      const existing = groups.get(dateKey) || [];
      groups.set(dateKey, [...existing, message]);
    }

    return groups;
  }

  const messageGroups = $derived(groupMessagesByDate(currentMessages));
</script>

<svelte:window
        on:resize={() => {
        /* handle resize */
    }}
/>

<svelte:head>
  <title>{selectedChannel ? `#${selectedChannel.name}` : "Chat"} | App</title>
  <meta name="description" content="Real-time chat application" />
</svelte:head>

<div
        class="chat-container {className}"
        class:dark={theme === "dark"}
        class:light={theme === "light"}
        data-testid="chat-container"
        {...restProps}
>
  <!-- Sidebar -->
  <aside class="sidebar" class:collapsed={!showUserList}>
    <header class="sidebar-header">
      <h2>Channels</h2>
      <button
              type="button"
              onclick={() => (showUserList = !showUserList)}
              aria-label={showUserList ? "Hide sidebar" : "Show sidebar"}
      >
        {#if showUserList}
          <span>←</span>
        {:else}
          <span>→</span>
        {/if}
      </button>
    </header>

    <div class="search-container">
      <input
              id="search-input"
              type="search"
              placeholder="Search users..."
              bind:value={searchQuery}
              aria-label="Search users and channels"
      />
    </div>

    <nav class="channel-list" aria-label="Channels">
      {#each sortedChannels as channel (channel.id)}
        {@const unread = unreadCounts[channel.id] || 0}
        {@const isSelected = selectedChannel?.id === channel.id}
        <button
                type="button"
                class="channel-item"
                class:selected={isSelected}
                class:has-unread={unread > 0}
                onclick={() => handleChannelClick(channel)}
                aria-current={isSelected ? "page" : undefined}
        >
                    <span class="channel-icon">
                        {#if channel.type === "voice"}
                            🔊
                        {:else if channel.type === "announcement"}
                            📢
                        {:else}
                            #
                        {/if}
                    </span>
          <span class="channel-name">{channel.name}</span>
          {#if unread > 0}
                        <span
                                class="unread-badge"
                                aria-label="{unread} unread messages"
                        >
                            {unread > 99 ? "99+" : unread}
                        </span>
          {/if}
        </button>
      {:else}
        <p class="empty-state">No channels available</p>
      {/each}
    </nav>

    <div class="user-sections">
      <section class="user-section">
        <h3>Online — {onlineUsers.length}</h3>
        <ul class="user-list">
          {#each onlineUsers as user (user.id)}
            <li>
              <button
                      type="button"
                      class="user-item"
                      onclick={() => onUserClick?.(user)}
              >
                <img
                        src={user.avatar}
                        alt=""
                        class="avatar"
                        width="32"
                        height="32"
                        loading="lazy"
                />
                <span class="user-name">{user.name}</span>
                <span
                        class="status-indicator {getStatusColor(
                                        user.status,
                                    )}"
                />
              </button>
            </li>
          {/each}
        </ul>
      </section>

      {#if awayUsers.length > 0}
        <section class="user-section">
          <h3>Away — {awayUsers.length}</h3>
          <ul class="user-list">
            {#each awayUsers as user (user.id)}
              <li>
                <button
                        type="button"
                        class="user-item"
                        onclick={() => onUserClick?.(user)}
                >
                  <img
                          src={user.avatar}
                          alt=""
                          class="avatar"
                  />
                  <span class="user-name">{user.name}</span>
                  <span
                          class="status-indicator {getStatusColor(
                                            user.status,
                                        )}"
                  />
                </button>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if offlineUsers.length > 0}
        <section class="user-section">
          <h3>Offline — {offlineUsers.length}</h3>
          <ul class="user-list">
            {#each offlineUsers as user (user.id)}
              <li>
                <button
                        type="button"
                        class="user-item"
                        onclick={() => onUserClick?.(user)}
                >
                  <img
                          src={user.avatar}
                          alt=""
                          class="avatar"
                  />
                  <span class="user-name">{user.name}</span>
                  <span
                          class="status-indicator {getStatusColor(
                                            user.status,
                                        )}"
                  />
                </button>
              </li>
            {/each}
          </ul>
        </section>
      {/if}
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    {#if selectedChannel}
      <header class="channel-header">
        <h1>#{selectedChannel.name}</h1>
        <p class="channel-description">{selectedChannel.description}</p>
        <div class="header-actions">
                    <span class="member-count"
                    >{selectedChannel.members.length} members</span
                    >
          {#if pinnedMessages.length > 0}
            <button type="button" class="pinned-btn">
              📌 {pinnedMessages.length} pinned
            </button>
          {/if}
        </div>
      </header>

      <div
              class="messages-container"
              bind:this={scrollContainer}
              role="log"
              aria-live="polite"
      >
        {#each [...messageGroups] as [dateKey, messages] (dateKey)}
          <div class="date-separator">
            <span>{formatDate(new Date(dateKey))}</span>
          </div>

          {#each messages as message, index (message.id)}
            {@const isOwn = message.author.id === currentUser.id}
            {@const showAvatar =
                    index === 0 ||
                    messages[index - 1]?.author.id !==
                    message.author.id}

            <article
                    class="message"
                    class:own={isOwn}
                    class:pinned={message.pinned}
                    class:edited={message.edited}
                    bind:this={
                                messageRefs.set(message.id, this) && undefined
                            }
                    data-message-id={message.id}
            >
              {#if showAvatar}
                <img
                        src={message.author.avatar}
                        alt="{message.author.name}'s avatar"
                        class="message-avatar"
                        width="40"
                        height="40"
                />
              {:else}
                <div class="avatar-placeholder" />
              {/if}

              <div class="message-content">
                {#if showAvatar}
                  <header class="message-header">
                                        <span class="author-name"
                                        >{message.author.name}</span
                                        >
                    <time
                            datetime={message.timestamp.toISOString()}
                    >
                      {formatTimestamp(message.timestamp)}
                    </time>
                    {#if message.edited}
                                            <span class="edited-indicator"
                                            >(edited)</span
                                            >
                    {/if}
                  </header>
                {/if}

                <p class="message-text">{message.content}</p>

                {#if message.attachments.length > 0}
                  <div class="attachments">
                    {#each message.attachments as attachment (attachment.id)}
                      {#if attachment.type === "image"}
                        <img
                                src={attachment.url}
                                alt={attachment.name}
                                class="attachment-image"
                                loading="lazy"
                        />
                      {:else if attachment.type === "video"}
                        <video
                                src={attachment.url}
                                controls
                                class="attachment-video"
                                preload="metadata"
                        >
                          <track kind="captions" />
                        </video>
                      {:else if attachment.type === "audio"}
                        <audio
                                src={attachment.url}
                                controls
                                class="attachment-audio"
                        >
                          <track kind="captions" />
                        </audio>
                      {:else}
                        <a
                                href={attachment.url}
                                class="attachment-file"
                                download={attachment.name}
                        >
                                                    <span class="file-icon"
                                                    >📎</span
                                                    >
                          <span class="file-name"
                          >{attachment.name}</span
                          >
                          <span class="file-size"
                          >{formatFileSize(
                                  attachment.size,
                          )}</span
                          >
                        </a>
                      {/if}
                    {/each}
                  </div>
                {/if}

                {#if message.reactions.length > 0}
                  <div class="reactions">
                    {#each message.reactions as reaction (reaction.emoji)}
                      <button
                              type="button"
                              class="reaction"
                              class:reacted={reaction.users.some(
                                                    (u) =>
                                                        u.id === currentUser.id,
                                                )}
                              onclick={() =>
                                                    handleReaction(
                                                        message.id,
                                                        reaction.emoji,
                                                    )}
                              title={reaction.users
                                                    .map((u) => u.name)
                                                    .join(", ")}
                      >
                                                <span class="emoji"
                                                >{reaction.emoji}</span
                                                >
                        <span class="count"
                        >{reaction.count}</span
                        >
                      </button>
                    {/each}
                  </div>
                {/if}

                {#if message.thread && message.thread.length > 0}
                  <details class="thread">
                    <summary>
                      {message.thread.length}
                      {message.thread.length === 1
                              ? "reply"
                              : "replies"}
                    </summary>
                    <div class="thread-messages">
                      {#each message.thread as reply (reply.id)}
                        <div class="reply">
                          <img
                                  src={reply.author
                                                            .avatar}
                                  alt=""
                                  class="reply-avatar"
                          />
                          <span class="reply-author"
                          >{reply.author
                                  .name}</span
                          >
                          <span class="reply-content"
                          >{reply.content}</span
                          >
                        </div>
                      {/each}
                    </div>
                  </details>
                {/if}
              </div>
            </article>
          {/each}
        {/each}

        {#if typingUsers.length > 0}
          <div class="typing-indicator" aria-live="polite">
            {#if typingUsers.length === 1}
              <span>{typingUsers[0].name} is typing...</span>
            {:else if typingUsers.length === 2}
                            <span
                            >{typingUsers[0].name} and {typingUsers[1].name} are
                                typing...</span
                            >
            {:else}
              <span>Several people are typing...</span>
            {/if}
          </div>
        {/if}
      </div>

      <footer class="message-input-container">
        <form
                onsubmit={(e) => {
                        e.preventDefault();
                        handleSendMessage();
                    }}
        >
          <div class="input-wrapper">
            <button
                    type="button"
                    class="emoji-btn"
                    onclick={() => (showEmojiPicker = !showEmojiPicker)}
                    aria-label="Open emoji picker"
            >
              😀
            </button>

            <input
                    type="text"
                    placeholder="Message #{selectedChannel.name}"
                    bind:value={messageInput}
                    oninput={() => {
                                isTyping = true;
                                dispatch("typing", {
                                    userId: currentUser.id,
                                    channelId: selectedChannel!.id,
                                });
                            }}
                    aria-label="Message input"
            />

            <button
                    type="submit"
                    disabled={!messageInput.trim()}
                    aria-label="Send message"
            >
              Send
            </button>
          </div>
        </form>

        {#if showEmojiPicker}
          <div
                  class="emoji-picker"
                  role="dialog"
                  aria-label="Emoji picker"
          >
            {#each ["😀", "😂", "❤️", "👍", "👎", "🎉", "🔥", "💯", "🤔", "😢", "😡", "🙏"] as emoji}
              <button
                      type="button"
                      onclick={() => {
                                    messageInput += emoji;
                                    showEmojiPicker = false;
                                }}
              >
                {emoji}
              </button>
            {/each}
          </div>
        {/if}
      </footer>
    {:else}
      <div class="no-channel-selected">
        <h2>Welcome!</h2>
        <p>Select a channel from the sidebar to start chatting.</p>

        {#if hasUnread}
          <p class="unread-notice">
            You have unread messages in some channels.
          </p>
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  .chat-container {
    display: grid;
    grid-template-columns: 280px 1fr;
    height: 100vh;
    font-family:
            system-ui,
            -apple-system,
            sans-serif;
  }

  .chat-container.dark {
    --bg-primary: #1a1a2e;
    --bg-secondary: #16213e;
    --text-primary: #eee;
    --text-secondary: #aaa;
    --border-color: #333;
    --accent: #0f4c75;
  }

  .chat-container.light {
    --bg-primary: #fff;
    --bg-secondary: #f5f5f5;
    --text-primary: #333;
    --text-secondary: #666;
    --border-color: #ddd;
    --accent: #3282b8;
  }

  .sidebar {
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar.collapsed {
    width: 60px;
  }

  .sidebar-header {
    padding: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
  }

  .search-container {
    padding: 0.5rem 1rem;
  }

  .search-container input {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .channel-list {
    flex: 0 0 auto;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .channel-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border: none;
    background: transparent;
    color: var(--text-primary);
    width: 100%;
    text-align: left;
    border-radius: 4px;
    cursor: pointer;
  }

  .channel-item:hover {
    background: var(--bg-primary);
  }

  .channel-item.selected {
    background: var(--accent);
    color: white;
  }

  .channel-item.has-unread .channel-name {
    font-weight: bold;
  }

  .unread-badge {
    background: #e74c3c;
    color: white;
    padding: 0.125rem 0.375rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    margin-left: auto;
  }

  .user-sections {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .user-section h3 {
    font-size: 0.75rem;
    text-transform: uppercase;
    color: var(--text-secondary);
    padding: 0.5rem;
  }

  .user-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .user-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border: none;
    background: transparent;
    color: var(--text-primary);
    width: 100%;
    border-radius: 4px;
    cursor: pointer;
  }

  .user-item:hover {
    background: var(--bg-primary);
  }

  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    object-fit: cover;
  }

  .status-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin-left: auto;
  }

  .main-content {
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .channel-header {
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .channel-header h1 {
    margin: 0;
    font-size: 1.25rem;
  }

  .channel-description {
    margin: 0.25rem 0 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .header-actions {
    display: flex;
    gap: 1rem;
    margin-top: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .date-separator {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 1rem 0;
    color: var(--text-secondary);
    font-size: 0.75rem;
  }

  .date-separator::before,
  .date-separator::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--border-color);
  }

  .message {
    display: flex;
    gap: 0.75rem;
    padding: 0.25rem 0;
  }

  .message.pinned {
    background: rgba(255, 215, 0, 0.1);
    padding: 0.5rem;
    border-radius: 4px;
  }

  .message-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    object-fit: cover;
    flex-shrink: 0;
  }

  .avatar-placeholder {
    width: 40px;
    flex-shrink: 0;
  }

  .message-content {
    flex: 1;
    min-width: 0;
  }

  .message-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .author-name {
    font-weight: 600;
  }

  .message-header time {
    color: var(--text-secondary);
    font-size: 0.75rem;
  }

  .edited-indicator {
    color: var(--text-secondary);
    font-size: 0.75rem;
    font-style: italic;
  }

  .message-text {
    margin: 0;
    word-wrap: break-word;
  }

  .attachments {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .attachment-image {
    max-width: 300px;
    max-height: 200px;
    border-radius: 4px;
  }

  .attachment-video,
  .attachment-audio {
    max-width: 100%;
  }

  .attachment-file {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
    text-decoration: none;
    color: var(--text-primary);
  }

  .reactions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-top: 0.5rem;
  }

  .reaction {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.125rem 0.375rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 9999px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .reaction.reacted {
    border-color: var(--accent);
    background: rgba(15, 76, 117, 0.2);
  }

  .thread {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
  }

  .thread summary {
    cursor: pointer;
    color: var(--accent);
    font-size: 0.875rem;
  }

  .thread-messages {
    margin-top: 0.5rem;
    padding-left: 0.5rem;
    border-left: 2px solid var(--border-color);
  }

  .reply {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0;
    font-size: 0.875rem;
  }

  .reply-avatar {
    width: 20px;
    height: 20px;
    border-radius: 50%;
  }

  .reply-author {
    font-weight: 600;
  }

  .typing-indicator {
    padding: 0.5rem;
    color: var(--text-secondary);
    font-size: 0.875rem;
    font-style: italic;
  }

  .message-input-container {
    padding: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .input-wrapper {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .input-wrapper input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .input-wrapper button {
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 4px;
    background: var(--accent);
    color: white;
    cursor: pointer;
  }

  .input-wrapper button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .emoji-btn {
    background: transparent;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
  }

  .emoji-picker {
    position: absolute;
    bottom: 100%;
    left: 0;
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 0.25rem;
    padding: 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .emoji-picker button {
    padding: 0.5rem;
    border: none;
    background: transparent;
    font-size: 1.25rem;
    cursor: pointer;
    border-radius: 4px;
  }

  .emoji-picker button:hover {
    background: var(--bg-primary);
  }

  .no-channel-selected {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    color: var(--text-secondary);
  }

  .no-channel-selected h2 {
    color: var(--text-primary);
  }

  .unread-notice {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: rgba(231, 76, 60, 0.2);
    border-radius: 4px;
  }

  .empty-state {
    padding: 1rem;
    text-align: center;
    color: var(--text-secondary);
  }
</style>
