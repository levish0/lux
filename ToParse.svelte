<script lang="ts">
    import { Button } from '$lib/components/ui/button';
    import { Textarea } from '$lib/components/ui/textarea';
    import { Spinner } from '$lib/components/ui/spinner';
    import * as Dialog from '$lib/components/ui/dialog';

    type Props = {
        open: boolean;
        revisionNumber: number | null;
        reason: string;
        loading: boolean;
        onOpenChange: (open: boolean) => void;
        onReasonChange: (reason: string) => void;
        onConfirm: () => void;
    };

    let { open, revisionNumber, reason, loading, onOpenChange, onReasonChange, onConfirm }: Props =
        $props();

    function handleClose() {
        onOpenChange(false);
    }
</script>

<Dialog.Root {open} {onOpenChange}>
    <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
            <Dialog.Title>리비전 되돌리기</Dialog.Title>
            <Dialog.Description>
                {#if revisionNumber}
                    r{revisionNumber} 리비전으로 되돌립니다.
                {/if}
            </Dialog.Description>
        </Dialog.Header>
        <div class="space-y-4 py-4">
            <div class="space-y-2">
                <label for="revert-reason" class="text-sm font-medium">
                    되돌리기 사유 <span class="text-red-500">*</span>
                </label>
                <Textarea
                        id="revert-reason"
                        placeholder="되돌리기 사유를 입력하세요"
                        value={reason}
                        oninput={(e) => onReasonChange(e.currentTarget.value)}
                        rows={3}
                />
            </div>
        </div>
        <Dialog.Footer>
            <Button variant="outline" onclick={handleClose} disabled={loading}>취소</Button>
            <Button onclick={onConfirm} disabled={loading || !reason.trim()}>
                {#if loading}
                    <Spinner class="mr-2 size-4" />
                {/if}
                되돌리기
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
