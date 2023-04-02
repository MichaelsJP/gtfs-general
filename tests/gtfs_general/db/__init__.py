# @pytest.mark.parametrize(
#     'init_status,is_pending,is_confirmed,is_reserved,is_transport,is_completed,is_canceled,is_not_appeared',
#     [(init_status, _is_pending, _is_confirmed, _is_reserved, _is_transport, _is_completed, _is_canceled,
#       _is_not_appeared) \
#      for init_status in (
#          0, (models.Order.Status.PENDING.mask
#              | models.Order.Status.CONFIRMED.mask
#              | models.Order.Status.RESERVED.mask
#              | models.Order.Status.TRANSPORT.mask
#              | models.Order.Status.COMPLETED.mask
#              | models.Order.Status.CANCELLED.mask
#              | models.Order.Status.NOT_APPEARED.mask)) \
#      for _is_pending in (False, True) \
#      for _is_confirmed in (False, True) \
#      for _is_reserved in (False, True) \
#      for _is_transport in (False, True) \
#      for _is_completed in (False, True) \
#      for _is_canceled in (False, True) \
#      for _is_not_appeared in (False, True)])
