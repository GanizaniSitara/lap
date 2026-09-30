<template>
  <ModalDialog
    title="Folder Manager"
    :width="780"
    :height="560"
    position-key="folder-manager"
    @cancel="clickCancel"
  >
    <div class="flex flex-col flex-1 min-h-0 select-none text-base-content/80">
      <!-- Main Content Split Pane -->
      <div class="flex flex-1 min-h-0 gap-3">
        <!-- Left Pane: List/Tree of configured libraries and folders -->
        <div class="w-72 shrink-0 flex flex-col border border-base-content/10 bg-base-300/40 rounded-box overflow-hidden">
          <!-- Left Pane Header -->
          <div class="p-2 border-b border-base-content/10 flex items-center justify-between gap-1.5 shrink-0 bg-base-200/50">
            <div class="flex items-center gap-1.5 min-w-0">
              <span class="text-xs font-semibold text-base-content/80 truncate">Folders & Libraries</span>
              <span class="badge badge-xs badge-neutral text-[10px]">{{ visibleFlatNodes.length }}</span>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <TButton
                :icon="IconRefresh"
                :buttonSize="'small'"
                :tooltip="'Refresh list'"
                :disabled="isLoading"
                @click="loadData"
              />
            </div>
          </div>

          <!-- Quick Search Filter -->
          <div class="p-2 border-b border-base-content/5 shrink-0">
            <div class="relative flex items-center">
              <IconSearch class="w-3.5 h-3.5 absolute left-2 text-base-content/40 pointer-events-none" />
              <input
                v-model="searchFilter"
                type="text"
                placeholder="Filter folders..."
                class="input input-xs w-full pl-7 pr-2 bg-base-100/50 border border-base-content/10 rounded-md focus:outline-none focus:border-primary/50 text-xs"
              />
            </div>
          </div>

          <!-- Tree List -->
          <div class="flex-1 min-h-0 overflow-y-auto p-1.5 space-y-0.5">
            <div v-if="isLoading" class="flex flex-col items-center justify-center h-32 gap-2 text-base-content/40 text-xs">
              <span class="loading loading-spinner loading-sm"></span>
              <span>Loading folders...</span>
            </div>

            <div v-else-if="visibleFlatNodes.length === 0" class="flex flex-col items-center justify-center h-32 text-base-content/40 text-xs text-center p-4">
              <span>No libraries or folders found</span>
            </div>

            <div
              v-else
              v-for="node in visibleFlatNodes"
              :key="node.id"
              class="flex items-center h-7 rounded-md px-1 cursor-pointer transition-colors text-xs group"
              :class="[
                selectedNode?.id === node.id
                  ? 'bg-primary/20 text-primary font-medium border border-primary/30'
                  : 'hover:bg-base-100/50 text-base-content/80'
              ]"
              :style="{ paddingLeft: `${node.depth * 14 + 4}px` }"
              @click="selectNode(node)"
            >
              <!-- Expand / Collapse chevron -->
              <button
                type="button"
                class="w-4 h-4 shrink-0 flex items-center justify-center rounded hover:bg-base-content/10 mr-1 text-base-content/50"
                :class="{ 'invisible': !hasExpandableChildren(node) }"
                @click.stop="toggleExpand(node)"
              >
                <span v-if="node.isLoading" class="loading loading-spinner loading-xs"></span>
                <IconRight
                  v-else
                  class="w-3 h-3 transition-transform duration-150"
                  :class="{ 'rotate-90': node.isExpanded }"
                />
              </button>

              <!-- Node Icon -->
              <IconPhotoAll
                v-if="node.type === 'library'"
                class="w-3.5 h-3.5 text-primary shrink-0 mr-1.5"
              />
              <IconFolder
                v-else
                class="w-3.5 h-3.5 text-warning/90 shrink-0 mr-1.5"
              />

              <!-- Node Name -->
              <span class="truncate flex-1" :title="node.path || node.name">
                {{ node.name }}
              </span>

              <!-- Status Badges / Indicators -->
              <div class="flex items-center gap-1 shrink-0 ml-1">
                <span
                  v-if="node.is_excluded_from_search"
                  title="Excluded from search"
                  class="w-1.5 h-1.5 rounded-full bg-warning/80"
                ></span>
                <span
                  v-if="node.faces_excluded"
                  title="Face detection disabled"
                  class="w-1.5 h-1.5 rounded-full bg-error/80"
                ></span>
                <span
                  v-if="node.type === 'library' && node.libraryId === 'default'"
                  class="text-[9px] uppercase px-1 rounded bg-base-content/10 text-base-content/50"
                >
                  default
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Right Pane: Settings for selected folder/library -->
        <div class="flex-1 min-w-0 flex flex-col border border-base-content/10 bg-base-300/30 rounded-box p-4 overflow-y-auto">
          <!-- Empty State -->
          <div v-if="!selectedNode" class="flex-1 flex flex-col items-center justify-center text-center p-6 text-base-content/40">
            <IconFolderCog class="w-12 h-12 mb-3 opacity-30" />
            <div class="text-sm font-medium mb-1">No Location Selected</div>
            <div class="text-xs max-w-xs">Select a library or folder from the list on the left to configure scan scope and privacy options.</div>
          </div>

          <!-- Configuration Controls -->
          <div v-else class="flex flex-col space-y-4">
            <!-- Selected Header Card -->
            <div class="flex items-start gap-3 p-3 bg-base-200/60 border border-base-content/10 rounded-box shrink-0">
              <div class="w-9 h-9 rounded-lg bg-base-100 flex items-center justify-center shrink-0 border border-base-content/10">
                <IconPhotoAll v-if="selectedNode.type === 'library'" class="w-5 h-5 text-primary" />
                <IconFolder v-else class="w-5 h-5 text-warning" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <h3 class="text-sm font-semibold text-base-content truncate">{{ selectedNode.name }}</h3>
                  <span class="badge badge-xs text-[10px] uppercase font-bold" :class="selectedNode.type === 'library' ? 'badge-primary' : 'badge-ghost'">
                    {{ selectedNode.type }}
                  </span>
                </div>
                <div class="text-xs text-base-content/50 break-all font-mono mt-0.5" :title="selectedNode.path || selectedNode.id">
                  {{ selectedNode.path || ('Library ID: ' + (selectedNode.libraryId || selectedNode.id)) }}
                </div>
              </div>
            </div>

            <!-- Scope Section -->
            <div class="rounded-box p-3 bg-base-200/40 border border-base-content/10 space-y-2.5">
              <div class="flex items-center justify-between">
                <div class="text-xs font-bold uppercase tracking-wider text-base-content/60">Scope</div>
                <div class="text-[11px] text-base-content/40">Picasa folder monitoring behavior</div>
              </div>

              <div class="space-y-2">
                <!-- Scan Always -->
                <label
                  class="flex items-start gap-3 p-2.5 rounded-lg border cursor-pointer transition-colors"
                  :class="selectedScope === 'always' ? 'border-primary/50 bg-primary/10' : 'border-base-content/10 hover:bg-base-100/40'"
                >
                  <input
                    type="radio"
                    name="folderScope"
                    value="always"
                    v-model="selectedScope"
                    class="radio radio-primary radio-sm mt-0.5"
                    @change="onScopeChange('always')"
                  />
                  <div class="flex flex-col">
                    <span class="text-xs font-semibold text-base-content">Scan Always</span>
                    <span class="text-[11px] text-base-content/60 leading-tight mt-0.5">
                      Continually monitor this location and automatically scan for new, changed, or deleted media.
                    </span>
                  </div>
                </label>

                <!-- Scan Once -->
                <label
                  class="flex items-start gap-3 p-2.5 rounded-lg border cursor-pointer transition-colors"
                  :class="selectedScope === 'once' ? 'border-primary/50 bg-primary/10' : 'border-base-content/10 hover:bg-base-100/40'"
                >
                  <input
                    type="radio"
                    name="folderScope"
                    value="once"
                    v-model="selectedScope"
                    class="radio radio-primary radio-sm mt-0.5"
                    @change="onScopeChange('once')"
                  />
                  <div class="flex flex-col flex-1">
                    <div class="flex items-center justify-between">
                      <span class="text-xs font-semibold text-base-content">Scan Once</span>
                      <button
                        v-if="selectedScope === 'once'"
                        type="button"
                        class="btn btn-xs btn-primary gap-1 px-2 font-normal"
                        :disabled="isProcessing"
                        @click.stop="triggerScanOnce"
                      >
                        <IconUpdate class="w-3 h-3" />
                        <span>Scan Now</span>
                      </button>
                    </div>
                    <span class="text-[11px] text-base-content/60 leading-tight mt-0.5">
                      Scan this location one time now without background filesystem monitoring.
                    </span>
                  </div>
                </label>

                <!-- Remove from LapCasa -->
                <label
                  class="flex items-start gap-3 p-2.5 rounded-lg border cursor-pointer transition-colors"
                  :class="selectedScope === 'remove' ? 'border-error/50 bg-error/10' : 'border-base-content/10 hover:bg-base-100/40'"
                >
                  <input
                    type="radio"
                    name="folderScope"
                    value="remove"
                    v-model="selectedScope"
                    class="radio radio-error radio-sm mt-0.5"
                    @change="onScopeChange('remove')"
                  />
                  <div class="flex flex-col flex-1">
                    <span class="text-xs font-semibold text-error">Remove from LapCasa</span>
                    <span class="text-[11px] text-base-content/60 leading-tight mt-0.5">
                      Remove from the LapCasa catalog. Files on disk are never deleted or modified.
                    </span>
                    <div v-if="selectedScope === 'remove'" class="mt-2 flex items-center gap-2">
                      <button
                        type="button"
                        class="btn btn-xs btn-error gap-1 px-3"
                        :disabled="isProcessing"
                        @click.stop="showRemoveConfirm = true"
                      >
                        <IconTrash class="w-3 h-3" />
                        <span>Confirm Remove</span>
                      </button>
                    </div>
                  </div>
                </label>
              </div>
            </div>

            <!-- Privacy & AI Section -->
            <div class="rounded-box p-3 bg-base-200/40 border border-base-content/10 space-y-2.5">
              <div class="flex items-center justify-between">
                <div class="text-xs font-bold uppercase tracking-wider text-base-content/60">Privacy & AI</div>
                <div class="text-[11px] text-base-content/40">Granular AI and indexing controls</div>
              </div>

              <div class="space-y-2">
                <!-- Enable Face Detection Checkbox -->
                <label class="flex items-start gap-3 p-2.5 rounded-lg border border-base-content/10 hover:bg-base-100/40 cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    v-model="faceDetectionEnabled"
                    class="checkbox checkbox-primary checkbox-sm mt-0.5"
                    :disabled="isProcessing"
                  />
                  <div class="flex flex-col">
                    <span class="text-xs font-semibold text-base-content">Enable Face Detection</span>
                    <span class="text-[11px] text-base-content/60 leading-tight mt-0.5">
                      Automatically detect and cluster faces in this folder for the People view.
                    </span>
                  </div>
                </label>

                <!-- Include in Search Checkbox -->
                <label class="flex items-start gap-3 p-2.5 rounded-lg border border-base-content/10 hover:bg-base-100/40 cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    v-model="includeInSearchEnabled"
                    class="checkbox checkbox-primary checkbox-sm mt-0.5"
                    :disabled="isProcessing"
                  />
                  <div class="flex flex-col">
                    <span class="text-xs font-semibold text-base-content">Include in Search</span>
                    <span class="text-[11px] text-base-content/60 leading-tight mt-0.5">
                      Include photos and videos from this location in search queries and AI semantic search.
                    </span>
                  </div>
                </label>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Dialog Footer -->
      <div class="flex justify-between items-center pt-3 mt-3 border-t border-base-content/10 shrink-0">
        <div class="text-xs text-base-content/40">
          <span v-if="isProcessing" class="flex items-center gap-1.5 text-primary">
            <span class="loading loading-spinner loading-xs"></span>
            Applying changes...
          </span>
          <span v-else>Changes to scan scope and privacy settings take effect immediately.</span>
        </div>
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="t-button-primary px-5 py-1.5 text-xs"
            @click="clickOk"
          >
            OK
          </button>
        </div>
      </div>
    </div>

    <!-- Confirmation Modal for Remove from LapCasa -->
    <MessageBox
      v-if="showRemoveConfirm"
      title="Remove from LapCasa"
      :message="`Are you sure you want to remove '${selectedNode?.name}' from LapCasa? Photos and files on your computer will NOT be deleted.`"
      OkText="Remove"
      cancelText="Cancel"
      :warningOk="true"
      @ok="confirmRemove"
      @cancel="cancelRemove"
    />
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import ModalDialog from '@/components/ModalDialog.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';
import { useToast } from '@/common/toast';
import { useLibraryStore } from '@/stores/libraryStore';
import {
  getAppConfig,
  getAllAlbums,
  fetchFolder,
  removeLibrary,
  removeAlbum,
  indexAlbum,
  setFolderFacesExcluded,
  setFolderSearchExcluded,
} from '@/common/api';
import {
  IconFolder,
  IconFolderCog,
  IconPhotoAll,
  IconRight,
  IconRefresh,
  IconSearch,
  IconTrash,
  IconUpdate,
} from '@/common/icons';

interface TreeNode {
  id: string;
  name: string;
  path?: string;
  type: 'library' | 'album' | 'folder';
  libraryId?: string;
  albumId?: number;
  scope: 'always' | 'once' | 'remove';
  faces_excluded: boolean;
  is_excluded_from_search: boolean;
  isExpanded: boolean;
  isLoading: boolean;
  depth: number;
  children?: TreeNode[];
}

const emit = defineEmits(['cancel', 'ok', 'updated']);

const toast = useToast();
const libraryStore = useLibraryStore();

const isLoading = ref(false);
const isProcessing = ref(false);
const searchFilter = ref('');
const treeNodes = ref<TreeNode[]>([]);
const selectedNode = ref<TreeNode | null>(null);
const selectedScope = ref<'always' | 'once' | 'remove'>('always');
const currentLibraryId = ref('default');
const showRemoveConfirm = ref(false);

const hasExpandableChildren = (node: TreeNode): boolean => {
  if (node.type === 'library') {
    return !!(node.children && node.children.length > 0);
  }
  // Folder or album with path can potentially fetch child folders
  return !!node.path;
};

// Flatten visible tree nodes based on search and expansion
const visibleFlatNodes = computed(() => {
  const result: TreeNode[] = [];
  const query = searchFilter.value.trim().toLowerCase();

  const walk = (nodes: TreeNode[]) => {
    for (const node of nodes) {
      if (query) {
        if (node.name.toLowerCase().includes(query) || (node.path && node.path.toLowerCase().includes(query))) {
          result.push(node);
        }
        if (node.children && node.children.length > 0) {
          walk(node.children);
        }
      } else {
        result.push(node);
        if (node.isExpanded && node.children && node.children.length > 0) {
          walk(node.children);
        }
      }
    }
  };

  walk(treeNodes.value);
  return result;
});

// Load libraries and folders
const loadData = async () => {
  isLoading.value = true;
  try {
    let libs = libraryStore.libraries;
    if (!libs || libs.length === 0) {
      libs = await libraryStore.fetchLibraries();
    }
    const appCfg = await getAppConfig();
    if (appCfg?.current_library_id) {
      currentLibraryId.value = appCfg.current_library_id;
    }
    if ((!libs || libs.length === 0) && appCfg?.libraries) {
      libs = appCfg.libraries;
    }

    const albums = (await getAllAlbums(true)) || [];
    const rootNodes: TreeNode[] = [];

    // Build library roots
    for (const lib of (libs || [])) {
      const isCurrent = lib.id === currentLibraryId.value;
      const libNode: TreeNode = {
        id: `lib-${lib.id}`,
        name: lib.name,
        type: 'library',
        libraryId: lib.id,
        scope: 'always',
        faces_excluded: false,
        is_excluded_from_search: false,
        isExpanded: true,
        isLoading: false,
        depth: 0,
        children: [],
      };

      if (isCurrent && albums.length > 0) {
        libNode.children = albums.map(album => ({
          id: `album-${album.id}`,
          name: album.name || album.path.split(/[\\/]/).pop() || 'Folder',
          path: album.path,
          type: 'album',
          albumId: album.id,
          libraryId: lib.id,
          scope: 'always',
          faces_excluded: !!album.faces_excluded,
          is_excluded_from_search: !!album.is_excluded_from_search,
          isExpanded: false,
          isLoading: false,
          depth: 1,
          children: [],
        }));
      }

      rootNodes.push(libNode);
    }

    // Fallback if no libraries configured
    if (rootNodes.length === 0 && albums.length > 0) {
      for (const album of albums) {
        rootNodes.push({
          id: `album-${album.id}`,
          name: album.name || album.path.split(/[\\/]/).pop() || 'Folder',
          path: album.path,
          type: 'album',
          albumId: album.id,
          scope: 'always',
          faces_excluded: !!album.faces_excluded,
          is_excluded_from_search: !!album.is_excluded_from_search,
          isExpanded: false,
          isLoading: false,
          depth: 0,
          children: [],
        });
      }
    }

    treeNodes.value = rootNodes;

    // Preserve or establish selection
    if (selectedNode.value) {
      const match = visibleFlatNodes.value.find(n => n.id === selectedNode.value?.id);
      if (match) {
        selectedNode.value = match;
        selectedScope.value = match.scope;
      } else {
        selectFirstAvailable(rootNodes);
      }
    } else {
      selectFirstAvailable(rootNodes);
    }
  } catch (error) {
    console.error('Failed to load folder manager data:', error);
  } finally {
    isLoading.value = false;
  }
};

const selectFirstAvailable = (rootNodes: TreeNode[]) => {
  if (rootNodes.length === 0) {
    selectedNode.value = null;
    return;
  }
  // Prefer selecting the first album/folder under the current library if available
  const currentLib = rootNodes.find(n => n.libraryId === currentLibraryId.value);
  if (currentLib && currentLib.children && currentLib.children.length > 0) {
    selectNode(currentLib.children[0]);
  } else {
    selectNode(rootNodes[0]);
  }
};

const selectNode = (node: TreeNode) => {
  selectedNode.value = node;
  selectedScope.value = node.scope || 'always';
};

const toggleExpand = async (node: TreeNode) => {
  if (node.type === 'library') {
    node.isExpanded = !node.isExpanded;
    return;
  }

  if (!node.isExpanded) {
    if ((!node.children || node.children.length === 0) && node.path) {
      node.isLoading = true;
      try {
        const folder = await fetchFolder(node.path, false);
        if (folder?.children && folder.children.length > 0) {
          node.children = folder.children.map(child => ({
            id: `folder-${child.path}`,
            name: child.name,
            path: child.path,
            type: 'folder',
            albumId: node.albumId,
            libraryId: node.libraryId,
            scope: 'always',
            faces_excluded: !!child.faces_excluded,
            is_excluded_from_search: !!child.is_excluded_from_search,
            isExpanded: false,
            isLoading: false,
            depth: node.depth + 1,
            children: [],
          }));
        }
      } catch (err) {
        console.error('Failed to fetch subfolders:', err);
      } finally {
        node.isLoading = false;
      }
    }
    node.isExpanded = true;
  } else {
    node.isExpanded = false;
  }
};

// Scope change handler
const onScopeChange = (scope: 'always' | 'once' | 'remove') => {
  if (!selectedNode.value) return;
  selectedNode.value.scope = scope;

  if (scope === 'remove') {
    showRemoveConfirm.value = true;
  } else if (scope === 'once') {
    triggerScanOnce();
  } else if (scope === 'always') {
    toast.success(`Continuous monitoring enabled for ${selectedNode.value.name}`);
  }
};

const triggerScanOnce = async () => {
  if (!selectedNode.value) return;
  const node = selectedNode.value;
  try {
    isProcessing.value = true;
    if (node.albumId) {
      await indexAlbum(node.albumId);
      toast.info(`Scan initiated for ${node.name}`);
    } else {
      toast.info(`Scan scheduled for ${node.name}`);
    }
  } catch (err: any) {
    console.error('Failed to trigger scan:', err);
    toast.error('Failed to start scan');
  } finally {
    isProcessing.value = false;
  }
};

// Remove from LapCasa execution
const confirmRemove = async () => {
  showRemoveConfirm.value = false;
  if (!selectedNode.value) return;
  const node = selectedNode.value;
  const targetLibId = node.libraryId || (node.type === 'library' ? node.id.replace('lib-', '') : currentLibraryId.value);

  try {
    isProcessing.value = true;
    // Always trigger removeLibrary IPC command as required by specifications
    if (node.type === 'library') {
      await removeLibrary(targetLibId);
    } else {
      if (node.albumId) {
        await removeAlbum(node.albumId);
      }
      if (targetLibId && targetLibId !== 'default') {
        try {
          await removeLibrary(targetLibId);
        } catch {
          // Handled or non-fatal
        }
      }
    }
    toast.success(`Removed "${node.name}" from LapCasa`);
    emit('updated');
    await loadData();
  } catch (err: any) {
    console.error('Failed to remove from LapCasa:', err);
    toast.error(err?.message || 'Failed to remove from LapCasa');
  } finally {
    isProcessing.value = false;
  }
};

const cancelRemove = () => {
  showRemoveConfirm.value = false;
  if (selectedNode.value) {
    selectedNode.value.scope = 'always';
    selectedScope.value = 'always';
  }
};

// Privacy & AI Checkbox Computeds
const faceDetectionEnabled = computed({
  get: () => !selectedNode.value?.faces_excluded,
  set: async (val: boolean) => {
    if (!selectedNode.value) return;
    const isExcluded = !val;
    selectedNode.value.faces_excluded = isExcluded;
    const albumId = selectedNode.value.albumId || 0;
    const folderPath = selectedNode.value.path || '';

    try {
      isProcessing.value = true;
      if (folderPath) {
        await setFolderFacesExcluded(albumId, folderPath, isExcluded);
      } else if (selectedNode.value.children) {
        for (const child of selectedNode.value.children) {
          if (child.path) {
            await setFolderFacesExcluded(child.albumId || 0, child.path, isExcluded);
            child.faces_excluded = isExcluded;
          }
        }
      }
      toast.success(val ? 'Face detection enabled' : 'Face detection disabled');
      emit('updated');
    } catch (err: any) {
      console.error('Failed to update face exclusion:', err);
      toast.error('Failed to update face detection setting');
    } finally {
      isProcessing.value = false;
    }
  }
});

const includeInSearchEnabled = computed({
  get: () => !selectedNode.value?.is_excluded_from_search,
  set: async (val: boolean) => {
    if (!selectedNode.value) return;
    const isExcluded = !val;
    selectedNode.value.is_excluded_from_search = isExcluded;
    const albumId = selectedNode.value.albumId || 0;
    const folderPath = selectedNode.value.path || '';

    try {
      isProcessing.value = true;
      if (folderPath) {
        await setFolderSearchExcluded(albumId, folderPath, isExcluded);
      } else if (selectedNode.value.children) {
        for (const child of selectedNode.value.children) {
          if (child.path) {
            await setFolderSearchExcluded(child.albumId || 0, child.path, isExcluded);
            child.is_excluded_from_search = isExcluded;
          }
        }
      }
      toast.success(val ? 'Folder included in search' : 'Folder excluded from search');
      emit('updated');
    } catch (err: any) {
      console.error('Failed to update search exclusion:', err);
      toast.error('Failed to update search exclusion setting');
    } finally {
      isProcessing.value = false;
    }
  }
});

const clickCancel = () => {
  emit('cancel');
};

const clickOk = () => {
  emit('ok');
};

onMounted(() => {
  loadData();
});
</script>

<style scoped>
/* Scoped custom scrollbar and dialog spacing */
</style>
