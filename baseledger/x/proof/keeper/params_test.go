package keeper_test

import (
	"testing"

	testkeeper "github.com/Baseledger/baseledger/testutil/keeper"
	"github.com/Baseledger/baseledger/x/proof/types"
	"github.com/stretchr/testify/require"
)

func TestGetParams(t *testing.T) {
	k, ctx := testkeeper.BaseledgerKeeper(t)
	params := types.DefaultParams()

	k.SetParams(ctx, params)

	require.EqualValues(t, params, k.GetParams(ctx))
}