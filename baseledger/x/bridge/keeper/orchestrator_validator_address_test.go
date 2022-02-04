package keeper_test

import (
	"strconv"
	"testing"

	"github.com/Baseledger/baseledger/x/bridge/keeper"
	"github.com/Baseledger/baseledger/x/bridge/types"
	keepertest "github.com/Baseledger/baseledger/testutil/keeper"
	"github.com/Baseledger/baseledger/testutil/nullify"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/require"
)

// Prevent strconv unused error
var _ = strconv.IntSize

func createNOrchestratorValidatorAddress(keeper *keeper.Keeper, ctx sdk.Context, n int) []types.OrchestratorValidatorAddress {
	items := make([]types.OrchestratorValidatorAddress, n)
	for i := range items {
		items[i].OrchestratorAddress = strconv.Itoa(i)
        
		keeper.SetOrchestratorValidatorAddress(ctx, items[i])
	}
	return items
}

func TestOrchestratorValidatorAddressGet(t *testing.T) {
	keeper, ctx := keepertest.BridgeKeeper(t)
	items := createNOrchestratorValidatorAddress(keeper, ctx, 10)
	for _, item := range items {
		rst, found := keeper.GetOrchestratorValidatorAddress(ctx,
		    item.OrchestratorAddress,
            
		)
		require.True(t, found)
		require.Equal(t,
			nullify.Fill(&item),
			nullify.Fill(&rst),
		)
	}
}
func TestOrchestratorValidatorAddressRemove(t *testing.T) {
	keeper, ctx := keepertest.BridgeKeeper(t)
	items := createNOrchestratorValidatorAddress(keeper, ctx, 10)
	for _, item := range items {
		keeper.RemoveOrchestratorValidatorAddress(ctx,
		    item.OrchestratorAddress,
            
		)
		_, found := keeper.GetOrchestratorValidatorAddress(ctx,
		    item.OrchestratorAddress,
            
		)
		require.False(t, found)
	}
}

func TestOrchestratorValidatorAddressGetAll(t *testing.T) {
	keeper, ctx := keepertest.BridgeKeeper(t)
	items := createNOrchestratorValidatorAddress(keeper, ctx, 10)
	require.ElementsMatch(t,
		nullify.Fill(items),
		nullify.Fill(keeper.GetAllOrchestratorValidatorAddress(ctx)),
	)
}